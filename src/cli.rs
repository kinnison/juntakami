//! Commandline support for Jan Takeshi

use std::path::{Path, PathBuf};

use clap::Parser;

mod helpers;
use git_testament::git_testament;
use helpers::*;

git_testament!(pub TESTAMENT);

#[derive(Parser)]
pub struct Cli {
    #[clap(short, long, env = "JUNTAKAMI_PATH", default_value_os_t = default_journal_path())]
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
    /// Show the status of the git tree in the journal
    Status,
    /// Show the configuration of the journal
    ShowConfig,
    /// Prepare today's entry
    Prep(PrepArgs),
    /// Edit today's entry
    Edit(PrepArgs),
}

#[derive(Clone, Parser)]
/// Initialise a journal folder
pub struct InitArgs {
    #[clap(short, long)]
    /// Force initialisation even if there's a journal config already
    force: bool,
}

impl InitArgs {
    pub fn force(&self) -> bool {
        self.force
    }
}

#[derive(Clone, Parser)]
pub struct PrepArgs {
    #[clap(short, long)]
    /// Log to edit (see config for default if not specified)
    prefix: Option<String>,
}

impl PrepArgs {
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
}
