use std::{ffi::OsStr, path::Path};

use clap::Parser;
use cli::Cmd;
use git_testament::GitModification;
use journal::NascentJournal;

mod cli;
mod journal;

fn main() {
    let args = cli::Cli::parse();
    let journal = NascentJournal::new(args.path());

    match args.cmd() {
        Cmd::DumpTestament => {
            dump_testament();
            std::process::exit(0);
        }
        Cmd::Init(args) => {
            args.initialise(journal);
            std::process::exit(0);
        }
        _ => {}
    }
    let journal = journal.load();

    match args.cmd() {
        Cmd::DumpTestament => {}
        Cmd::Init(_) => {}
        Cmd::Status => journal.show_status(),
    }
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
