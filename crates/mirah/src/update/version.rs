use semver::Version;

const TAG_KEY: &str = "\"tag_name\"";
const TAG_PREFIX: char = 'v';

fn tags(json: &str) -> impl Iterator<Item = &str> {
    json.split(TAG_KEY).skip(1).filter_map(|rest| {
        let rest = rest.trim_start().strip_prefix(':')?.trim_start();
        let rest = rest.strip_prefix('"')?;
        rest.split('"').next()
    })
}

pub(crate) fn parse(tag: &str) -> Option<Version> {
    Version::parse(tag.strip_prefix(TAG_PREFIX).unwrap_or(tag)).ok()
}

pub(crate) fn newest(json: &str, current: &Version) -> Option<Version> {
    let stable_only = current.pre.is_empty();
    tags(json)
        .filter_map(parse)
        .filter(|v| !stable_only || v.pre.is_empty())
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;

    const RELEASES: &str = r#"[
      {"url": "x", "tag_name": "v0.1.0-alpha.9", "prerelease": true},
      {"tag_name":"v0.1.0-alpha.10","prerelease":true},
      {"tag_name": "v0.1.0-alpha.2"},
      {"tag_name": "not-a-version"}
    ]"#;

    fn v(s: &str) -> Version {
        parse(s).expect("valid version")
    }

    #[test]
    fn finds_the_newest_tag_by_semver_not_by_position() {
        assert_eq!(
            newest(RELEASES, &v("0.1.0-alpha.1")),
            Some(v("0.1.0-alpha.10"))
        );
    }

    #[test]
    fn a_stable_install_is_only_offered_stable_releases() {
        let releases = r#"[{"tag_name":"v0.4.1-alpha.1"},{"tag_name":"v0.4.0"},{"tag_name":"v0.4.0-alpha.11"}]"#;
        assert_eq!(newest(releases, &v("0.4.0")), Some(v("0.4.0")));
        assert_eq!(
            newest(releases, &v("0.4.0-alpha.11")),
            Some(v("0.4.1-alpha.1"))
        );
    }

    #[test]
    fn a_final_release_is_newer_than_its_prereleases() {
        assert!(v("0.1.0") > v("0.1.0-alpha.10"));
        assert!(v("v0.1.0-alpha.10") > v("0.1.0-alpha.9"));
    }

    #[test]
    fn reads_the_version_inside_a_tag() {
        assert_eq!(parse("v0.1.0-alpha.8"), Some(v("0.1.0-alpha.8")));
        assert_eq!(parse("0.2.0"), Some(v("0.2.0")));
        assert_eq!(parse("latest"), None);
    }

    #[test]
    fn an_empty_or_broken_answer_has_no_version() {
        let now = v("0.1.0");
        assert_eq!(newest("[]", &now), None);
        assert_eq!(newest(r#"{"message": "Not Found"}"#, &now), None);
        assert_eq!(newest(r#"{"tag_name": 5}"#, &now), None);
    }
}
