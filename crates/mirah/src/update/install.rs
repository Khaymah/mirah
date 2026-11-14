use super::source;
use crate::chrome::update::Step;
use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);
const CHECKSUM_SUFFIX: &str = ".sha256";
const OLD_SUFFIX: &str = ".old";
const NEW_SUFFIX: &str = ".new";
const BINARY: &str = "mirah";
const WORK_DIR_ATTEMPTS: u32 = 16;

fn binary_name() -> String {
    format!("{BINARY}{}", std::env::consts::EXE_SUFFIX)
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub(crate) fn expected_hash(checksum_file: &str) -> Option<String> {
    let hash = checksum_file
        .split_whitespace()
        .next()?
        .to_ascii_lowercase();
    (hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())).then_some(hash)
}

fn verify(archive: &Path, checksum: &Path) -> Result<()> {
    let expected = std::fs::read_to_string(checksum)
        .ok()
        .and_then(|text| expected_hash(&text))
        .context("the checksum file is missing or unreadable")?;
    let actual = sha256_hex(&std::fs::read(archive).context("cannot read the download")?);
    if actual != expected {
        bail!("the download does not match its SHA-256 checksum, nothing was installed");
    }
    Ok(())
}

fn tar() -> Command {
    source::system_tool("tar")
}

fn extract(archive: &Path, into: &Path) -> Result<()> {
    std::fs::create_dir_all(into)?;
    let output = tar()
        .arg("-xf")
        .arg(archive)
        .arg("-C")
        .arg(into)
        .stdin(Stdio::null())
        .output()
        .context("could not start tar")?;
    if !output.status.success() {
        let reason = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!("tar could not unpack the download: {reason}");
    }
    Ok(())
}

fn find(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() {
            if let Some(found) = find(&path, name) {
                return Some(found);
            }
        } else if kind.is_file() && path.file_name().is_some_and(|n| n == name) {
            return Some(path);
        }
    }
    None
}

fn with_suffix(current: &Path, suffix: &str) -> PathBuf {
    let mut name = current.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    current.with_file_name(name)
}

fn old_path(current: &Path) -> PathBuf {
    with_suffix(current, OLD_SUFFIX)
}

pub(crate) fn replace(current: &Path, new: &Path) -> Result<()> {
    let staged = with_suffix(current, NEW_SUFFIX);
    let _ = std::fs::remove_file(&staged);
    if let Err(err) = std::fs::copy(new, &staged) {
        let _ = std::fs::remove_file(&staged);
        return Err(err).context("cannot write the new mirah, the old one was kept");
    }
    let old = old_path(current);
    let _ = std::fs::remove_file(&old);
    if let Err(err) = std::fs::rename(current, &old) {
        let _ = std::fs::remove_file(&staged);
        return Err(err).context("cannot move the running mirah aside");
    }
    if let Err(err) = std::fs::rename(&staged, current) {
        let _ = std::fs::rename(&old, current);
        let _ = std::fs::remove_file(&staged);
        return Err(err).context("cannot put the new mirah in place, the old one was kept");
    }
    Ok(())
}

fn work_dir() -> Result<PathBuf> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    for attempt in 0..WORK_DIR_ATTEMPTS {
        let dir = std::env::temp_dir().join(format!(
            "mirah-update-{}-{nanos:x}-{attempt}",
            std::process::id()
        ));
        match std::fs::create_dir(&dir) {
            Ok(()) => return Ok(dir),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(err) => return Err(err).context("cannot make a folder for the download"),
        }
    }
    bail!("cannot make a folder for the download")
}

pub(crate) fn remove_leftover() {
    if let Ok(current) = std::env::current_exe() {
        let _ = std::fs::remove_file(old_path(&current));
        let _ = std::fs::remove_file(with_suffix(&current, NEW_SUFFIX));
    }
}

pub(crate) fn install(tag: &str) -> Result<()> {
    install_with(tag, &mut |_, _| {})
}

pub(crate) fn install_with(tag: &str, report: &mut dyn FnMut(Step, Option<u8>)) -> Result<()> {
    let artifact = source::artifact().context("there is no release build for this platform")?;
    let current = std::env::current_exe().context("cannot find the running mirah")?;
    let work = work_dir()?;
    let result = (|| {
        report(Step::Download, None);
        let archive = work.join(artifact);
        let checksum = work.join(format!("{artifact}{CHECKSUM_SUFFIX}"));
        source::download_watching(
            &source::download_url(tag, artifact),
            &archive,
            DOWNLOAD_TIMEOUT,
            &mut |percent| report(Step::Download, Some(percent)),
        )?;
        let checksum_url = source::download_url(tag, &format!("{artifact}{CHECKSUM_SUFFIX}"));
        source::download(&checksum_url, &checksum, DOWNLOAD_TIMEOUT)?;
        report(Step::Verify, None);
        verify(&archive, &checksum)?;
        report(Step::Install, None);
        let unpacked = work.join("unpacked");
        extract(&archive, &unpacked)?;
        let binary = find(&unpacked, &binary_name()).context("the download has no mirah binary")?;
        replace(&current, &binary)
    })();
    let _ = std::fs::remove_dir_all(&work);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mirah-install-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn hashes_and_reads_checksum_files() {
        let hash = sha256_hex(b"abc");
        assert_eq!(
            hash,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            expected_hash(&format!("{} *mirah.zip\n", hash.to_uppercase())),
            Some(hash)
        );
        assert_eq!(expected_hash("nope"), None);
    }

    #[test]
    fn a_mismatched_download_is_rejected() {
        let dir = scratch("verify");
        let archive = dir.join("a.zip");
        let checksum = dir.join("a.zip.sha256");
        std::fs::write(&archive, b"abc").expect("write");
        std::fs::write(&checksum, format!("{}  a.zip", sha256_hex(b"abc"))).expect("write");
        assert!(verify(&archive, &checksum).is_ok());
        std::fs::write(&archive, b"abd").expect("write");
        assert!(verify(&archive, &checksum).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn replacing_keeps_the_old_binary_aside() {
        let dir = scratch("replace");
        let current = dir.join("mirah");
        let new = dir.join("new-mirah");
        std::fs::write(&current, b"old").expect("write");
        std::fs::write(&new, b"new").expect("write");
        replace(&current, &new).expect("replace");
        assert_eq!(std::fs::read(&current).expect("read"), b"new");
        assert_eq!(std::fs::read(old_path(&current)).expect("read"), b"old");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_failed_copy_puts_the_old_binary_back() {
        let dir = scratch("restore");
        let current = dir.join("mirah");
        std::fs::write(&current, b"old").expect("write");
        assert!(replace(&current, &dir.join("missing")).is_err());
        assert_eq!(std::fs::read(&current).expect("read"), b"old");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn replacing_leaves_no_staged_copy_and_each_download_gets_its_own_folder() {
        let dir = scratch("staged");
        let current = dir.join("mirah");
        let new = dir.join("new-mirah");
        std::fs::write(&current, b"old").expect("write");
        std::fs::write(&new, b"new").expect("write");
        replace(&current, &new).expect("replace");
        assert!(!with_suffix(&current, NEW_SUFFIX).exists());
        let (a, b) = (work_dir().expect("dir"), work_dir().expect("dir"));
        assert_ne!(a, b);
        let _ = std::fs::remove_dir_all(a);
        let _ = std::fs::remove_dir_all(b);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn a_link_to_a_binary_elsewhere_is_not_followed() {
        let dir = scratch("link");
        let outside = scratch("outside");
        std::fs::write(outside.join(binary_name()), b"bin").expect("write");
        std::os::unix::fs::symlink(&outside, dir.join("nested")).expect("link");
        assert_eq!(find(&dir, &binary_name()), None);
        let _ = std::fs::remove_dir_all(dir);
        let _ = std::fs::remove_dir_all(outside);
    }

    #[test]
    fn finds_the_binary_inside_a_nested_folder() {
        let dir = scratch("find");
        let nested = dir.join("mirah-x86_64-unknown-linux-gnu");
        std::fs::create_dir_all(&nested).expect("mkdir");
        std::fs::write(nested.join(binary_name()), b"bin").expect("write");
        assert_eq!(find(&dir, &binary_name()), Some(nested.join(binary_name())));
        let _ = std::fs::remove_dir_all(dir);
    }
}
