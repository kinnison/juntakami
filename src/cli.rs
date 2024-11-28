//! Commandline support for Jan Takeshi

use std::path::{Path, PathBuf};

use clap::Parser;

mod helpers;
use git_testament::git_testament;
use helpers::*;

use crate::journal::NascentJournal;

git_testament!(pub TESTAMENT);

#[derive(Parser)]
pub struct Cli {
    #[clap(short, long, env = "JANTAKESHI_PATH", default_value_os_t = default_journal_path())]
    /// The path to the journal
    path: PathBuf,
    #[clap(subcommand)]
    cmd: Cmd,
}

impl Cli {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn cmd(&self) -> &Cmd {
        &self.cmd
    }
}

#[derive(Clone, Parser)]
pub enum Cmd {
    #[clap(hide = true)]
    DumpTestament,
    Init(InitArgs),
    Status,
}

impl Cmd {
    pub fn needs_existing(&self) -> bool {
        !matches!(self, Self::DumpTestament | Self::Init(_))
    }
}

#[derive(Clone, Parser)]
/// Initialise a journal folder
pub struct InitArgs {}

impl InitArgs {
    pub fn initialise(&self, journal: NascentJournal) {
        todo!()
    }
}
