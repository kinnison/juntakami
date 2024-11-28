//! Helpers for the JT CLI

use std::path::{Path, PathBuf};

pub fn default_journal_path() -> PathBuf {
    home::home_dir()
        .unwrap_or(Path::new(std::path::Component::RootDir.as_os_str()).into())
        .join("journal")
}
