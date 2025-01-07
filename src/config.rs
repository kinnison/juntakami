//! Configuration for journals

use std::path::Path;

use eyre::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use time::format_description::OwnedFormatItem;

#[derive(Default, Serialize, Deserialize)]
struct RawConfiguration {
    juntakami: RawDefaults,
    log_meta: RawLogMeta,
}

#[derive(Serialize, Deserialize)]
struct RawDefaults {
    default_log: String,
    logging_pattern: String,
    list_char: char,
    editor: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct RawLogMeta {
    title: String,
    created: String,
    author: String,
}

pub const CONFIG_FILENAME: &str = "juntakami.conf";
pub const JOURNAL_BASE: &str = "@JOURNAL@";
pub const JOURNAL_ENTRY: &str = "@ENTRY@";

impl Default for RawDefaults {
    fn default() -> Self {
        Self {
            default_log: "log".into(),
            logging_pattern: "[year]-[month]-[day].md".into(),
            list_char: '-',
            editor: ["code", JOURNAL_BASE, JOURNAL_ENTRY]
                .into_iter()
                .map(String::from)
                .collect(),
        }
    }
}

impl Default for RawLogMeta {
    fn default() -> Self {
        Self {
            title: "Daily log entry for [year]-[month]-[day]".into(),
            created: "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second] [offset_hour][offset_minute]".into(),
            author: whoami::realname(),
        }
    }
}

struct ParsedConfiguration {
    logging_pattern: OwnedFormatItem,
    log_meta: ParsedLogMeta,
}

struct ParsedLogMeta {
    title: OwnedFormatItem,
    created: OwnedFormatItem,
}

pub struct Configuration {
    raw: RawConfiguration,
    parsed: ParsedConfiguration,
}

impl ParsedLogMeta {
    fn parse(path: &Path, raw: &RawLogMeta) -> Result<Self> {
        let title = time::format_description::parse_owned::<2>(&raw.title).with_context(|| {
            format!(
                "Trying to parse log_meta.title `{}` from {}",
                raw.title,
                path.display()
            )
        })?;
        let created =
            time::format_description::parse_owned::<2>(&raw.created).with_context(|| {
                format!(
                    "Trying to parse log_meta.created `{}` from {}",
                    raw.created,
                    path.display()
                )
            })?;
        Ok(Self { title, created })
    }
}

impl ParsedConfiguration {
    fn parse(path: &Path, raw: &RawConfiguration) -> Result<Self> {
        let logging_pattern =
            time::format_description::parse_owned::<2>(&raw.juntakami.logging_pattern)
                .with_context(|| {
                    format!(
                        "Trying to parse log_pattern `{}` from {}",
                        raw.juntakami.logging_pattern,
                        path.display()
                    )
                })?;
        let log_meta = ParsedLogMeta::parse(path, &raw.log_meta)?;
        Ok(Self {
            logging_pattern,
            log_meta,
        })
    }
}

impl Default for Configuration {
    fn default() -> Self {
        let raw = RawConfiguration::default();
        let parsed = ParsedConfiguration::parse(Path::new(""), &raw).unwrap();
        Self { raw, parsed }
    }
}

impl Configuration {
    /// Log prefix to use by default
    pub fn default_log(&self) -> &str {
        &self.raw.juntakami.default_log
    }
    /// Log filename for a given date
    pub fn logging_pattern(&self) -> &OwnedFormatItem {
        &self.parsed.logging_pattern
    }

    /// The character to use for unordered lists
    pub fn list_char(&self) -> char {
        self.raw.juntakami.list_char
    }

    /// The title to give to new log entries
    pub fn title(&self) -> &OwnedFormatItem {
        &self.parsed.log_meta.title
    }

    /// The created text to give to new log entries
    pub fn created(&self) -> &OwnedFormatItem {
        &self.parsed.log_meta.created
    }

    /// The author to attribute new log entries to
    pub fn author(&self) -> &str {
        &self.raw.log_meta.author
    }

    /// Get the raw config text
    pub fn raw_config(&self) -> Result<String> {
        toml::to_string(&self.raw).context("Serialising configuration")
    }

    /// The editor to run for editing journal entries
    pub fn editor(&self) -> &[impl AsRef<str>] {
        &self.raw.juntakami.editor
    }

    /// Write to disk
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let toml = self.raw_config()?;
        std::fs::write(path, &toml)
            .with_context(|| format!("Attempting to save configuration to {}", path.display()))
    }

    /// Read from disk
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let toml = std::fs::read_to_string(path)
            .with_context(|| format!("Attempting to read configuration from {}", path.display()))?;
        let raw: RawConfiguration = toml::from_str(&toml)
            .with_context(|| format!("Parsing contents of {}", path.display()))?;
        let parsed = ParsedConfiguration::parse(path, &raw)?;
        // Some sanity checks...
        if raw.juntakami.editor.is_empty() {
            bail!("Editor is empty in {}", path.display());
        }
        if raw.juntakami.editor.iter().all(|v| v != JOURNAL_ENTRY) {
            bail!(
                "Editor command does not include `{JOURNAL_ENTRY}` anywhere, in {}",
                path.display()
            );
        }
        if raw.juntakami.editor[0].starts_with('@') {
            bail!(
                "Editor command cannot start with substitution variable `{}`, in {}",
                raw.juntakami.editor[0],
                path.display()
            );
        }
        Ok(Self { raw, parsed })
    }
}
