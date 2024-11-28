use std::{ffi::OsStr, path::Path};

use clap::Parser;
use cli::Cmd;
use eyre::Result;
use git_testament::GitModification;
use journal::NascentJournal;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

mod cli;
pub mod config;
mod git;
mod journal;

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var("JUNTAKAMI_LOG")
                .from_env_lossy(),
        )
        .init();
    let args = cli::Cli::parse();
    let journal = NascentJournal::new(args.path());

    match args.cmd() {
        Cmd::DumpTestament => {
            dump_testament();
            return Ok(());
        }
        Cmd::Init(args) => {
            journal.initialise(args)?;
            return Ok(());
        }
        _ => {}
    }
    let journal = journal.load()?;

    match args.cmd() {
        Cmd::DumpTestament => {}
        Cmd::Init(_) => {}
        Cmd::Status => journal.show_status()?,
    }

    Ok(())
}

fn dump_testament() {
    use cli::TESTAMENT;
    println!("Version displays as: {TESTAMENT}");
    println!(
        "Branch was {}",
        TESTAMENT.branch_name.unwrap_or("{unknown}")
    );
    for edit in TESTAMENT.modifications {
        // SAFETY: The unsafe{}s here are OK because we got the
        // filenames from the OS in the first place thanks to
        // git-testament
        match edit {
            GitModification::Added(raw_f) => {
                let fname = Path::new(unsafe { OsStr::from_encoded_bytes_unchecked(raw_f) });
                println!("Added: {}", fname.display());
            }
            GitModification::Removed(raw_f) => {
                let fname = Path::new(unsafe { OsStr::from_encoded_bytes_unchecked(raw_f) });
                println!("Removed: {}", fname.display());
            }
            GitModification::Modified(raw_f) => {
                let fname = Path::new(unsafe { OsStr::from_encoded_bytes_unchecked(raw_f) });
                println!("Modified: {}", fname.display());
            }
            GitModification::Untracked(raw_f) => {
                let fname = Path::new(unsafe { OsStr::from_encoded_bytes_unchecked(raw_f) });
                println!("Untracked: {}", fname.display());
            }
        }
    }
}
