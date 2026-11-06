use crate::command;

const LAST_FILE: &str = "last-agent";
pub(crate) const SHELL: &str = "shell";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Agent {
    pub(crate) name: String,
    pub(crate) title: String,
    pub(crate) command: String,
    pub(crate) resume: Vec<String>,
}

const KNOWN: [(&str, &str, &[&str]); 11] = [
    ("claude", "Claude Code", &["--continue"]),
    ("opencode", "OpenCode", &["--continue"]),
    ("codex", "Codex CLI", &["resume", "--last"]),
    ("gemini", "Gemini CLI", &["--resume"]),
    ("qwen", "Qwen Code", &[]),
    ("aider", "Aider", &[]),
    ("crush", "Crush", &[]),
    ("cursor-agent", "Cursor Agent", &[]),
    ("amp", "Amp", &[]),
    ("copilot", "Copilot CLI", &[]),
    ("goose", "Goose", &[]),
];

fn known() -> impl Iterator<Item = Agent> {
    KNOWN.iter().map(|(name, title, resume)| Agent {
        name: (*name).to_string(),
        title: (*title).to_string(),
        command: (*name).to_string(),
        resume: resume.iter().map(|s| (*s).to_string()).collect(),
    })
}

pub(crate) fn shell() -> Agent {
    Agent {
        name: SHELL.to_string(),
        title: crate::config::program_name(&command::shell()),
        command: command::shell(),
        resume: Vec::new(),
    }
}

pub(crate) fn detect(extra: &[Agent], installed: impl Fn(&str) -> bool) -> Vec<Agent> {
    let mut found: Vec<Agent> = extra.to_vec();
    for agent in known() {
        if !found.iter().any(|a| a.name == agent.name) && installed(&agent.command) {
            found.push(agent);
        }
    }
    found.push(shell());
    found
}

pub(crate) fn installed_agents(extra: &[Agent]) -> Vec<Agent> {
    detect(extra, command::installed)
}

pub(crate) fn find(name: &str, extra: &[Agent]) -> Option<Agent> {
    let key = crate::config::program_name(name);
    extra
        .iter()
        .cloned()
        .chain(known())
        .chain(std::iter::once(shell()))
        .find(|a| a.name == key)
}

pub(crate) fn resume_argv(agent: &Agent, argv: &[String]) -> Vec<String> {
    let mut out = vec![
        argv.first()
            .cloned()
            .unwrap_or_else(|| agent.command.clone()),
    ];
    out.extend(agent.resume.iter().cloned());
    out
}

pub(crate) fn last() -> Option<String> {
    let path = crate::paths::state_dir()?.join(LAST_FILE);
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
}

pub(crate) fn remember(name: &str) {
    if let Some(dir) = crate::paths::state_dir() {
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join(LAST_FILE), name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_only_installed_agents_then_the_shell() {
        let found = detect(&[], |name| name == "opencode" || name == "claude");
        let names: Vec<&str> = found.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["claude", "opencode", SHELL]);
    }

    #[test]
    fn agents_from_the_config_come_first_and_replace_known_ones() {
        let mine = Agent {
            name: "claude".into(),
            title: "Claude (work)".into(),
            command: "claude-work".into(),
            resume: vec![],
        };
        let found = detect(std::slice::from_ref(&mine), |_| true);
        assert_eq!(found[0], mine);
        assert_eq!(found.iter().filter(|a| a.name == "claude").count(), 1);
    }

    #[test]
    fn resuming_uses_each_program_s_own_flag() {
        let claude = find("C:\\npm\\claude.cmd", &[]).expect("known");
        assert_eq!(
            resume_argv(&claude, &["claude".into(), "--model".into()]),
            ["claude", "--continue"]
        );
        let codex = find("codex", &[]).expect("known");
        assert_eq!(resume_argv(&codex, &[]), ["codex", "resume", "--last"]);
        let aider = find("aider", &[]).expect("known");
        assert_eq!(resume_argv(&aider, &["aider".into()]), ["aider"]);
    }
}
