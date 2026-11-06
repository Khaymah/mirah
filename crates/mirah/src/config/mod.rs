mod keys;

pub(crate) use keys::{Key, SPECS, Spec, spec};
use mirah_term::{Align, InputOrder, RenderOptions, Shaping};
use std::path::PathBuf;
use toml_edit::{DocumentMut, Item, Table, value};

const PROGRAMS: &str = "programs";
const AGENTS: &str = "agents";
const SESSIONS_DIR: &str = "sessions_dir";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Choice<T> {
    Auto,
    Fixed(T),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Settings {
    pub(crate) align: Align,
    pub(crate) shaping: Choice<Shaping>,
    pub(crate) input: Choice<InputOrder>,
    pub(crate) tables: bool,
    pub(crate) mouse: bool,
    pub(crate) update_check: bool,
    pub(crate) bar: bool,
    pub(crate) record: bool,
    pub(crate) clipboard: bool,
    pub(crate) highlight: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            align: Align::Prose,
            shaping: Choice::Auto,
            input: Choice::Auto,
            tables: true,
            mouse: true,
            update_check: true,
            bar: true,
            record: false,
            clipboard: true,
            highlight: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Scope {
    Global,
    Program(String),
}

pub(crate) fn program_name(program: &str) -> String {
    let lower = program.to_lowercase();
    let name = lower.rsplit(['/', '\\']).next().unwrap_or_default();
    name.split('.').next().unwrap_or_default().to_string()
}

impl Settings {
    pub(crate) fn apply(&mut self, key: Key, text: &str) -> bool {
        let flag = || match text {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        };
        match key {
            Key::Align => match text {
                "prose" => self.align = Align::Prose,
                "arabic" => self.align = Align::Arabic,
                "off" => self.align = Align::Off,
                _ => return false,
            },
            Key::Shaping => match text {
                "auto" => self.shaping = Choice::Auto,
                "full" => self.shaping = Choice::Fixed(Shaping::Full),
                "plain" => self.shaping = Choice::Fixed(Shaping::Plain),
                _ => return false,
            },
            Key::Input => match text {
                "auto" => self.input = Choice::Auto,
                "logical" => self.input = Choice::Fixed(InputOrder::Logical),
                "visual" => self.input = Choice::Fixed(InputOrder::Visual),
                _ => return false,
            },
            Key::Tables => return flag().map(|f| self.tables = f).is_some(),
            Key::Mouse => return flag().map(|f| self.mouse = f).is_some(),
            Key::UpdateCheck => return flag().map(|f| self.update_check = f).is_some(),
            Key::Record => return flag().map(|f| self.record = f).is_some(),
            Key::Clipboard => return flag().map(|f| self.clipboard = f).is_some(),
            Key::Highlight => return flag().map(|f| self.highlight = f).is_some(),
        }
        true
    }

    pub(crate) fn value(&self, key: Key) -> &'static str {
        let flag = |f: bool| if f { "true" } else { "false" };
        match key {
            Key::Align => match self.align {
                Align::Prose => "prose",
                Align::Arabic => "arabic",
                Align::Off => "off",
            },
            Key::Shaping => match self.shaping {
                Choice::Auto => "auto",
                Choice::Fixed(Shaping::Full) => "full",
                Choice::Fixed(Shaping::Plain) => "plain",
            },
            Key::Input => match self.input {
                Choice::Auto => "auto",
                Choice::Fixed(InputOrder::Logical) => "logical",
                Choice::Fixed(InputOrder::Visual) => "visual",
            },
            Key::Tables => flag(self.tables),
            Key::Mouse => flag(self.mouse),
            Key::UpdateCheck => flag(self.update_check),
            Key::Record => flag(self.record),
            Key::Clipboard => flag(self.clipboard),
            Key::Highlight => flag(self.highlight),
        }
    }

    fn apply_table(&mut self, table: &Table) {
        for spec in &SPECS {
            let text = match table.get(spec.name) {
                Some(item) => item
                    .as_str()
                    .map(str::to_string)
                    .or_else(|| item.as_bool().map(|b| b.to_string())),
                None => None,
            };
            if let Some(text) = text
                && !self.apply(spec.key, &text)
            {
                tracing::warn!(key = spec.name, value = text, "ignored a bad setting");
            }
        }
        if let Some(bar) = table.get("bar").and_then(Item::as_bool) {
            self.bar = bar;
        }
    }

    fn apply_env(&mut self) {
        for (key, env) in [
            (Key::Align, "MIRAH_ALIGN"),
            (Key::Shaping, "MIRAH_SHAPING"),
            (Key::Input, "MIRAH_INPUT"),
        ] {
            if let Ok(text) = std::env::var(env) {
                self.apply(key, &text);
            }
        }
        if std::env::var_os("MIRAH_NO_UPDATE_CHECK").is_some() {
            self.update_check = false;
        }
        if std::env::var("MIRAH_BAR").is_ok_and(|v| v == "0") {
            self.bar = false;
        }
    }

    pub(crate) fn render_options(&self, program: &str) -> RenderOptions {
        RenderOptions {
            shaping: match self.shaping {
                Choice::Auto => crate::host::detected_shaping(),
                Choice::Fixed(s) => s,
            },
            align: self.align,
            input: match self.input {
                Choice::Auto => crate::host::detected_input_order(program, cfg!(windows)),
                Choice::Fixed(i) => i,
            },
            tables: self.tables,
            highlight: self.highlight,
        }
    }
}

pub(crate) struct Config {
    path: Option<PathBuf>,
    doc: DocumentMut,
    writable: bool,
}

impl Config {
    pub(crate) fn load() -> Self {
        Self::from_path(crate::paths::config_file())
    }

    pub(crate) fn from_path(path: Option<PathBuf>) -> Self {
        let text = path
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .unwrap_or_default();
        match text.parse::<DocumentMut>() {
            Ok(doc) => Self {
                path,
                doc,
                writable: true,
            },
            Err(err) => {
                tracing::warn!(error = %err, "config.toml is not valid, using defaults and not saving");
                Self {
                    path,
                    doc: DocumentMut::new(),
                    writable: false,
                }
            }
        }
    }

    fn program_table(&self, program: &str) -> Option<&Table> {
        self.doc
            .get(PROGRAMS)?
            .as_table()?
            .get(&program_name(program))?
            .as_table()
    }

    pub(crate) fn settings(&self, program: &str) -> Settings {
        let mut settings = Settings::default();
        settings.apply_table(self.doc.as_table());
        if let Some(table) = self.program_table(program) {
            settings.apply_table(table);
        }
        settings.apply_env();
        settings
    }

    pub(crate) fn sessions_dir(&self) -> Option<PathBuf> {
        self.doc
            .get(SESSIONS_DIR)
            .and_then(Item::as_str)
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from)
    }

    pub(crate) fn agents(&self) -> Vec<crate::agents::Agent> {
        let Some(list) = self.doc.get(AGENTS).and_then(Item::as_array_of_tables) else {
            return Vec::new();
        };
        list.iter()
            .filter_map(|table| {
                let text = |key: &str| table.get(key).and_then(Item::as_str).map(str::to_string);
                let command = text("command")?;
                let name = text("name").unwrap_or_else(|| program_name(&command));
                let resume = table
                    .get("resume")
                    .and_then(Item::as_array)
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default();
                Some(crate::agents::Agent {
                    title: text("title").unwrap_or_else(|| name.clone()),
                    name,
                    command,
                    resume,
                })
            })
            .collect()
    }

    pub(crate) fn set(&mut self, scope: &Scope, key: Key, text: &str) {
        let name = spec(key).name;
        let item = match text {
            "true" => value(true),
            "false" => value(false),
            other => value(other),
        };
        match scope {
            Scope::Global => {
                self.doc[name] = item;
            }
            Scope::Program(program) => {
                let programs = self
                    .doc
                    .entry(PROGRAMS)
                    .or_insert(Item::Table(Table::new()));
                if let Some(table) = programs.as_table_mut() {
                    table.set_implicit(true);
                    let entry = table
                        .entry(&program_name(program))
                        .or_insert(Item::Table(Table::new()));
                    entry[name] = item;
                }
            }
        }
    }

    pub(crate) fn save(&self) -> std::io::Result<()> {
        let Some(path) = self.path.as_ref().filter(|_| self.writable) else {
            return Ok(());
        };
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, self.doc.to_string())
    }

    #[cfg(test)]
    fn text(&self) -> String {
        self.doc.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(text: &str) -> Config {
        Config {
            path: None,
            doc: text.parse().expect("valid toml"),
            writable: true,
        }
    }

    #[test]
    fn a_program_section_overrides_the_global_settings() {
        let c =
            config("align = \"arabic\"\ntables = false\n[programs.opencode]\nalign = \"off\"\n");
        assert_eq!(c.settings("claude").align, Align::Arabic);
        assert!(!c.settings("claude").tables);
        assert_eq!(c.settings("C:\\npm\\OpenCode.cmd").align, Align::Off);
    }

    #[test]
    fn the_sessions_folder_can_be_set() {
        assert_eq!(
            config("sessions_dir = 'D:/mirah/sessions'\n").sessions_dir(),
            Some(PathBuf::from("D:/mirah/sessions"))
        );
        assert_eq!(config("").sessions_dir(), None);
    }

    #[test]
    fn agents_come_from_the_config() {
        let c = config(
            "[[agents]]\nname = \"work\"\ncommand = \"claude-work\"\nresume = [\"--continue\"]\n\n[[agents]]\ncommand = \"C:\\\\tools\\\\kimi.exe\"\n\n[[agents]]\nname = \"broken\"\n",
        );
        let agents = c.agents();
        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0].name, "work");
        assert_eq!(agents[0].resume, ["--continue"]);
        assert_eq!(agents[1].name, "kimi");
        assert_eq!(agents[1].title, "kimi");
    }

    #[test]
    fn bad_values_fall_back_to_the_default() {
        let c = config("align = \"sideways\"\nmouse = 3\n");
        assert_eq!(c.settings("x"), Settings::default());
    }

    #[test]
    fn setting_a_value_keeps_comments_and_other_keys() {
        let mut c = config("# my settings\nalign = \"arabic\"\n");
        c.set(&Scope::Program("claude.exe".into()), Key::Tables, "false");
        c.set(&Scope::Global, Key::Align, "off");
        let text = c.text();
        assert!(text.contains("# my settings"));
        assert!(text.contains("align = \"off\""));
        assert!(text.contains("[programs.claude]"));
        assert!(!c.settings("claude").tables);
        assert!(c.settings("codex").tables);
    }

    #[test]
    fn every_value_round_trips() {
        for spec in &SPECS {
            for (text, _) in spec.values {
                let mut s = Settings::default();
                assert!(s.apply(spec.key, text));
                assert_eq!(s.value(spec.key), *text);
            }
        }
    }

    #[test]
    fn an_invalid_file_is_never_overwritten() {
        let dir = std::env::temp_dir().join(format!("mirah-config-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let path = dir.join("config.toml");
        std::fs::write(&path, "align = [broken").expect("write");
        let mut c = Config::from_path(Some(path.clone()));
        c.set(&Scope::Global, Key::Align, "off");
        c.save().expect("save is a no-op");
        assert_eq!(
            std::fs::read_to_string(&path).expect("read"),
            "align = [broken"
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
