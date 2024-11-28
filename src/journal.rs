//! The journal itself

use std::path::{Path, PathBuf};

pub struct NascentJournal {
    base: PathBuf,
}

pub struct Journal {
    base: PathBuf,
}
impl Journal {
    pub fn show_status(&self) {
        todo!()
    }
}

impl NascentJournal {
    /// Construct a Journal
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            base: path.as_ref().into(),
        }
    }

    /// Acquire the journal config
    pub fn load(self) -> Journal {
        let Self { base } = self;
        Journal { base }
    }
}
