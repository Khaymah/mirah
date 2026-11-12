use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use toml_edit::{DocumentMut, value};

const SESSIONS_DIR: &str = "sessions";
const INSTALL_ROOT: &str = ".mirah";
const INSTALL_BIN: &str = "bin";
const GIT_IGNORE: &str = ".gitignore";
const GIT_IGNORE_ALL: &str = "*\n";
const META_FILE: &str = "session.toml";
const OUTPUT_FILE: &str = "output.vt";
const LOG_FILE: &str = "mirah.log";
const KEEP_COUNT: usize = 50;
const KEEP_FOR: Duration = Duration::from_secs(14 * 24 * 60 * 60);
const ID_ALPHABET: &[u8] = b"abcdefghijkmnpqrstuvwxyz23456789";
const ID_SUFFIX: usize = 4;
const SECS_PER_DAY: u64 = 86_400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SessionMeta {
    pub(crate) id: String,
    pub(crate) agent: String,
    pub(crate) argv: Vec<String>,
    pub(crate) cwd: PathBuf,
    pub(crate) started: u64,
    pub(crate) ended: Option<u64>,
}

pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn civil(secs: u64) -> (u64, u64, u64, u64) {
    let days = secs / SECS_PER_DAY;
    let rem = secs % SECS_PER_DAY;
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (month, day, rem / 3600, rem % 3600 / 60)
}

pub(crate) fn new_id(secs: u64, salt: u64) -> String {
    let (month, day, hour, minute) = civil(secs);
    let mut seed = salt ^ secs.rotate_left(17) ^ 0x9e37_79b9_7f4a_7c15;
    let suffix: String = (0..ID_SUFFIX)
        .map(|_| {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let index = (seed >> 33) as usize % ID_ALPHABET.len();
            char::from(ID_ALPHABET[index])
        })
        .collect();
    format!("{month:02}{day:02}-{hour:02}{minute:02}-{suffix}")
}

fn looks_like_id(name: &str) -> bool {
    let parts: Vec<&str> = name.split('-').collect();
    parts.len() == 3
        && parts[0].len() == 4
        && parts[1].len() == 4
        && parts[2].len() == ID_SUFFIX
        && parts[0]
            .chars()
            .chain(parts[1].chars())
            .all(|c| c.is_ascii_digit())
}

pub(crate) struct Store {
    root: PathBuf,
}

impl Store {
    pub(crate) fn open() -> Option<Self> {
        let custom = crate::config::Config::load().sessions_dir();
        let beside_app = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(beside));
        let fallback = crate::paths::state_dir().map(|dir| dir.join(SESSIONS_DIR));
        let store = custom
            .into_iter()
            .chain(beside_app)
            .find(|dir| writable(dir))
            .or(fallback)
            .map(Self::at)?;
        store.keep_out_of_git();
        Some(store)
    }

    fn keep_out_of_git(&self) {
        let ignore = self.root.join(GIT_IGNORE);
        if self.root.is_dir() && !ignore.exists() {
            let _ = std::fs::write(ignore, GIT_IGNORE_ALL);
        }
    }

    pub(crate) fn at(root: PathBuf) -> Self {
        Self { root }
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn dir(&self, id: &str) -> PathBuf {
        self.root.join(id)
    }

    pub(crate) fn output(&self, id: &str) -> PathBuf {
        self.dir(id).join(OUTPUT_FILE)
    }

    pub(crate) fn log(&self, id: &str) -> PathBuf {
        self.dir(id).join(LOG_FILE)
    }

    pub(crate) fn reserve(&self) -> String {
        let started = now();
        let salt = u64::from(std::process::id()) ^ u64::from(std::process::id()).rotate_left(29);
        let mut id = new_id(started, salt);
        let mut attempt = 0;
        while self.dir(&id).exists() {
            attempt += 1;
            id = new_id(started, salt.wrapping_add(attempt));
        }
        id
    }

    #[cfg(test)]
    pub(crate) fn create(&self, agent: &str, argv: &[String], cwd: &Path) -> SessionMeta {
        self.create_with(None, agent, argv, cwd)
    }

    pub(crate) fn create_with(
        &self,
        id: Option<String>,
        agent: &str,
        argv: &[String],
        cwd: &Path,
    ) -> SessionMeta {
        let started = now();
        let meta = SessionMeta {
            id: id.unwrap_or_else(|| self.reserve()),
            agent: agent.to_string(),
            argv: argv.to_vec(),
            cwd: cwd.to_path_buf(),
            started,
            ended: None,
        };
        self.save(&meta);
        meta
    }

    pub(crate) fn save(&self, meta: &SessionMeta) {
        let dir = self.dir(&meta.id);
        if let Err(err) = std::fs::create_dir_all(&dir) {
            tracing::warn!(error = %err, "cannot create the session folder");
            return;
        }
        let mut doc = DocumentMut::new();
        doc["id"] = value(meta.id.as_str());
        doc["agent"] = value(meta.agent.as_str());
        let mut argv = toml_edit::Array::new();
        meta.argv.iter().for_each(|a| argv.push(a.as_str()));
        doc["argv"] = value(argv);
        doc["cwd"] = value(meta.cwd.to_string_lossy().as_ref());
        doc["started"] = value(i64::try_from(meta.started).unwrap_or(0));
        if let Some(ended) = meta.ended {
            doc["ended"] = value(i64::try_from(ended).unwrap_or(0));
        }
        if let Err(err) = std::fs::write(dir.join(META_FILE), doc.to_string()) {
            tracing::warn!(error = %err, "cannot save session.toml");
        }
    }

    pub(crate) fn load(&self, id: &str) -> Option<SessionMeta> {
        let text = std::fs::read_to_string(self.dir(id).join(META_FILE)).ok()?;
        let doc: DocumentMut = text.parse().ok()?;
        let number = |key: &str| {
            doc.get(key)
                .and_then(|v| v.as_integer())
                .and_then(|n| u64::try_from(n).ok())
        };
        Some(SessionMeta {
            id: doc.get("id")?.as_str()?.to_string(),
            agent: doc.get("agent")?.as_str()?.to_string(),
            argv: doc
                .get("argv")?
                .as_array()?
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect(),
            cwd: PathBuf::from(doc.get("cwd")?.as_str()?),
            started: number("started")?,
            ended: number("ended"),
        })
    }

    pub(crate) fn list(&self) -> Vec<SessionMeta> {
        let mut all: Vec<SessionMeta> = std::fs::read_dir(&self.root)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .filter_map(|e| self.load(&e.file_name().to_string_lossy()))
                    .collect()
            })
            .unwrap_or_default();
        all.sort_by_key(|m| std::cmp::Reverse(m.started));
        all
    }

    pub(crate) fn clean(&self, now: u64) -> usize {
        let Ok(entries) = std::fs::read_dir(&self.root) else {
            return 0;
        };
        let mut found: Vec<(u64, PathBuf)> = entries
            .filter_map(Result::ok)
            .filter(|e| looks_like_id(&e.file_name().to_string_lossy()))
            .map(|e| {
                let id = e.file_name().to_string_lossy().into_owned();
                let started = self.load(&id).map_or(0, |m| m.started);
                (started, e.path())
            })
            .collect();
        found.sort_by_key(|f| std::cmp::Reverse(f.0));
        let cutoff = now.saturating_sub(KEEP_FOR.as_secs());
        let mut removed = 0;
        for (index, (started, path)) in found.into_iter().enumerate() {
            if index >= KEEP_COUNT && started < cutoff && std::fs::remove_dir_all(&path).is_ok() {
                removed += 1;
            }
        }
        removed
    }
}

fn beside(app_dir: &Path) -> PathBuf {
    let installed = app_dir.file_name().is_some_and(|n| n == INSTALL_BIN)
        && app_dir
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|n| n == INSTALL_ROOT);
    match (installed, app_dir.parent()) {
        (true, Some(root)) => root.join(SESSIONS_DIR),
        _ => app_dir.join(SESSIONS_DIR),
    }
}

fn writable(dir: &Path) -> bool {
    if std::fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(format!(".mirah-write-{}", std::process::id()));
    let ok = std::fs::write(&probe, b"").is_ok();
    let _ = std::fs::remove_file(&probe);
    ok
}

pub(crate) fn forget_if_empty(store: &Store, id: &str) {
    let dir = store.dir(id);
    if !dir.join(META_FILE).exists() {
        let _ = std::fs::remove_file(dir.join(LOG_FILE));
        let _ = std::fs::remove_dir(dir);
    }
}

pub(crate) fn age(now: u64, then: u64) -> String {
    let secs = now.saturating_sub(then);
    match secs {
        s if s < 60 => "now".to_string(),
        s if s < 3_600 => format!("{}m ago", s / 60),
        s if s < SECS_PER_DAY => format!("{}h ago", s / 3_600),
        s => format!("{}d ago", s / SECS_PER_DAY),
    }
}

pub(crate) fn human_size(bytes: u64) -> String {
    const KB: u64 = 1_024;
    match bytes {
        0 => "-".to_string(),
        b if b < KB => format!("{b} B"),
        b if b < KB * KB => format!("{} KB", b / KB),
        b => format!("{:.1} MB", b as f64 / (KB * KB) as f64),
    }
}

pub(crate) fn table(store: &Store, now: u64) -> String {
    let sessions = store.list();
    if sessions.is_empty() {
        return format!("No sessions yet.\nFolder: {}", store.root().display());
    }
    let mut out = String::from("ID              AGENT       WHEN      RECORDING  FOLDER\n");
    for meta in sessions {
        out.push_str(&format!(
            "{:<15} {:<11} {:<9} {:<10} {}\n",
            meta.id,
            meta.agent,
            age(now, meta.started),
            human_size(size_of(&store.output(&meta.id))),
            meta.cwd.display()
        ));
    }
    out.push_str(&format!(
        "\nmirah resume <ID> continues a session.\nFolder: {}",
        store.root().display()
    ));
    out
}

pub(crate) fn size_of(path: &Path) -> u64 {
    std::fs::metadata(path).map_or(0, |m| m.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sessions_sit_beside_the_app_but_not_inside_the_install_bin_folder() {
        let home = Path::new("/home/me");
        assert_eq!(
            beside(&home.join(".mirah").join("bin")),
            home.join(".mirah").join("sessions")
        );
        assert_eq!(
            beside(&home.join("tools")),
            home.join("tools").join("sessions")
        );
        assert_eq!(beside(&home.join("bin")), home.join("bin").join("sessions"));
    }

    fn temp(name: &str) -> Store {
        let root = std::env::temp_dir().join(format!("mirah-store-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        Store::at(root)
    }

    #[test]
    fn ids_are_short_readable_and_dated() {
        let id = new_id(1_793_934_190, 7);
        assert!(looks_like_id(&id), "{id}");
        assert!(id.starts_with("1106-"), "{id}");
        assert_ne!(new_id(1_793_934_190, 8), id);
    }

    #[test]
    fn a_session_round_trips_through_its_folder() {
        let store = temp("roundtrip");
        let argv = vec![
            "claude".to_string(),
            "--model".to_string(),
            "x y".to_string(),
        ];
        let mut meta = store.create("claude", &argv, Path::new("D:\\Work\\Test App"));
        assert_eq!(store.load(&meta.id), Some(meta.clone()));
        meta.ended = Some(meta.started + 5);
        store.save(&meta);
        assert_eq!(
            store.load(&meta.id).and_then(|m| m.ended),
            Some(meta.started + 5)
        );
        assert_eq!(store.list().len(), 1);
        let _ = std::fs::remove_dir_all(store.root());
    }

    #[test]
    fn cleaning_keeps_recent_and_the_newest_fifty() {
        let store = temp("clean");
        let now = 2_000_000_000;
        for i in 0..60u64 {
            let id = format!("0101-0000-{:04}", i).replace('0', "a");
            let id = format!("0101-{:04}-{}", i, &id[10..]);
            let meta = SessionMeta {
                id,
                agent: "claude".into(),
                argv: vec![],
                cwd: PathBuf::new(),
                started: if i < 5 { now } else { 1000 + i },
                ended: None,
            };
            store.save(&meta);
        }
        let removed = store.clean(now);
        assert_eq!(removed, 10);
        assert_eq!(store.list().len(), 50);
        let _ = std::fs::remove_dir_all(store.root());
    }
}
