//! Configuration for journals

use std::path::Path;

use eyre::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    log_pattern: String,
    list_char: char,
}

pub const CONFIG_FILENAME: &str = "juntakami.conf";

impl Default for Configuration {
    fn default() -> Self {
        Self {
            log_pattern: "log/[year]-[month]-[day].md".into(),
            list_char: '-',
        }
    }
}

impl Configuration {
    /// Log filename for a given date
    pub fn log_pattern(&self) -> &str {
        &self.log_pattern
    }
    /// The character to use for unordered lists
    pub fn list_char(&self) -> char {
        self.list_char
    }

    /// Write to disk
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let toml = toml::to_string(self).context("Serialising configuration to save it")?;
        std::fs::write(path, &toml)
            .with_context(|| format!("Attempting to save configuration to {}", path.display()))
    }

    /// Read from disk
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let toml = std::fs::read_to_string(path)
            .with_context(|| format!("Attempting to read configuration from {}", path.display()))?;
        toml::from_str(&toml).with_context(|| format!("Parsing contents of {}", path.display()))
    }
}
