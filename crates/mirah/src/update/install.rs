use super::source;
use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);
const CHECKSUM_SUFFIX: &str = ".sha256";
const OLD_SUFFIX: &str = ".old";
const BINARY: &str = "mirah";

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

#[cfg(windows)]
fn tar() -> Command {
    let system = std::env::var_os("SystemRoot").map(PathBuf::from);
    match system.map(|root| root.join("System32").join("tar.exe")) {
        Some(path) if path.is_file() => Command::new(path),
        _ => Command::new("tar"),
    }
}

#[cfg(not(windows))]
fn tar() -> Command {
    Command::new("tar")
}

fn extract(archive: &Path, into: &Path) -> Result<()> {
    std::fs::create_dir_all(into)?;
    let status = tar()
        .arg("-xf")
        .arg(archive)
        .arg("-C")
        .arg(into)
        .status()
        .context("could not start tar")?;
    if !status.success() {
        bail!("tar could not unpack the download");
    }
    Ok(())
}

fn find(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find(&path, name) {
                return Some(found);
            }
        } else if path.file_name().is_some_and(|n| n == name) {
            return Some(path);
        }
    }
    None
}

fn old_path(current: &Path) -> PathBuf {
    let mut name = current.file_name().unwrap_or_default().to_os_string();
    name.push(OLD_SUFFIX);
    current.with_file_name(name)
}

pub(crate) fn replace(current: &Path, new: &Path) -> Result<()> {
    let old = old_path(current);
    let _ = std::fs::remove_file(&old);
    std::fs::rename(current, &old).context("cannot move the running mirah aside")?;
    if let Err(err) = std::fs::copy(new, current) {
        let _ = std::fs::rename(&old, current);
        return Err(err).context("cannot write the new mirah, the old one was kept");
    }
    Ok(())
}

pub(crate) fn remove_leftover() {
    if let Ok(current) = std::env::current_exe() {
        let _ = std::fs::remove_file(old_path(&current));
    }
}

pub(crate) fn install(tag: &str) -> Result<()> {
    let artifact = source::artifact().context("there is no release build for this platform")?;
    let current = std::env::current_exe().context("cannot find the running mirah")?;
    let work = std::env::temp_dir().join(format!("mirah-update-{}", std::process::id()));
    std::fs::create_dir_all(&work)?;
    let result = (|| {
        let archive = work.join(artifact);
        let checksum = work.join(format!("{artifact}{CHECKSUM_SUFFIX}"));
        source::download(
            &source::download_url(tag, artifact),
            &archive,
            DOWNLOAD_TIMEOUT,
        )?;
        let checksum_url = source::download_url(tag, &format!("{artifact}{CHECKSUM_SUFFIX}"));
        source::download(&checksum_url, &checksum, DOWNLOAD_TIMEOUT)?;
        verify(&archive, &checksum)?;
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
    fn finds_the_binary_inside_a_nested_folder() {
        let dir = scratch("find");
        let nested = dir.join("mirah-x86_64-unknown-linux-gnu");
        std::fs::create_dir_all(&nested).expect("mkdir");
        std::fs::write(nested.join(binary_name()), b"bin").expect("write");
        assert_eq!(find(&dir, &binary_name()), Some(nested.join(binary_name())));
        let _ = std::fs::remove_dir_all(dir);
    }
}
