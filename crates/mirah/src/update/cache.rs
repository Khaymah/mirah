use semver::Version;
use std::path::Path;
use std::time::Duration;

const CHECK_TTL: Duration = Duration::from_secs(60 * 60);
const DISMISS_TTL: Duration = Duration::from_secs(24 * 60 * 60);
const NONE: &str = "-";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Cache {
    pub(crate) checked_at: u64,
    pub(crate) latest: Option<Version>,
    pub(crate) dismissed: Option<(Version, u64)>,
}

fn version(text: &str) -> Option<Version> {
    (text != NONE).then(|| Version::parse(text).ok()).flatten()
}

impl Cache {
    pub(crate) fn parse(text: &str) -> Self {
        let mut cache = Self::default();
        for line in text.lines() {
            let mut parts = line.split_whitespace();
            match (parts.next(), parts.next(), parts.next()) {
                (Some("checked"), Some(at), None) => cache.checked_at = at.parse().unwrap_or(0),
                (Some("latest"), Some(v), None) => cache.latest = version(v),
                (Some("dismissed"), Some(v), Some(at)) => {
                    cache.dismissed = version(v).zip(at.parse().ok());
                }
                _ => {}
            }
        }
        cache
    }

    pub(crate) fn render(&self) -> String {
        let mut out = format!(
            "checked {}\nlatest {}\n",
            self.checked_at,
            self.latest
                .as_ref()
                .map_or_else(|| NONE.to_string(), Version::to_string)
        );
        if let Some((v, at)) = &self.dismissed {
            out.push_str(&format!("dismissed {v} {at}\n"));
        }
        out
    }

    pub(crate) fn fresh(&self, now: u64) -> bool {
        now.saturating_sub(self.checked_at) < CHECK_TTL.as_secs()
    }

    pub(crate) fn offer(&self, current: &Version, now: u64) -> Option<Version> {
        let latest = self.latest.as_ref().filter(|latest| *latest > current)?;
        let dismissed = self
            .dismissed
            .as_ref()
            .is_some_and(|(v, at)| v == latest && now.saturating_sub(*at) < DISMISS_TTL.as_secs());
        (!dismissed).then(|| latest.clone())
    }

    pub(crate) fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .map(|t| Self::parse(&t))
            .unwrap_or_default()
    }

    pub(crate) fn save(&self, path: &Path) {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, self.render());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        Version::parse(s).expect("valid version")
    }

    #[test]
    fn round_trips_through_text() {
        let cache = Cache {
            checked_at: 100,
            latest: Some(v("0.1.0-alpha.9")),
            dismissed: Some((v("0.1.0-alpha.9"), 90)),
        };
        assert_eq!(Cache::parse(&cache.render()), cache);
        assert_eq!(Cache::parse("garbage"), Cache::default());
    }

    #[test]
    fn a_check_is_fresh_for_an_hour() {
        let cache = Cache {
            checked_at: 1000,
            ..Cache::default()
        };
        assert!(cache.fresh(1000 + 3599));
        assert!(!cache.fresh(1000 + 3600));
    }

    #[test]
    fn offers_a_newer_version_unless_it_was_dismissed_today() {
        let current = v("0.1.0-alpha.8");
        let mut cache = Cache {
            checked_at: 0,
            latest: Some(v("0.1.0-alpha.9")),
            dismissed: None,
        };
        assert_eq!(cache.offer(&current, 10), Some(v("0.1.0-alpha.9")));
        cache.dismissed = Some((v("0.1.0-alpha.9"), 10));
        assert_eq!(cache.offer(&current, 20), None);
        assert_eq!(
            cache.offer(&current, 10 + 24 * 3600),
            Some(v("0.1.0-alpha.9"))
        );
        cache.latest = Some(v("0.1.0-alpha.10"));
        assert_eq!(cache.offer(&current, 20), Some(v("0.1.0-alpha.10")));
        assert_eq!(cache.offer(&v("0.1.0-alpha.10"), 20), None);
    }
}
