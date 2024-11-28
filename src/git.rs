//! Interfacing to running Git as a subprocess

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

use eyre::{bail, Context, Result};
use tracing::warn;

pub struct Git {
    available: bool,
    base: PathBuf,
}

impl Git {
    pub fn new(base: impl AsRef<Path>) -> Self {
        let base = base.as_ref().to_path_buf();
        let available = match Command::new("git")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
        {
            Ok(b) => b,
            Err(e) => {
                warn!("Unable to run git: {e}");
                false
            }
        };
        if !available {
            warn!("All git operations will be stubbed");
        }
        Self { base, available }
    }

    pub fn _git<I, S>(&self, args: I) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut cmd = Command::new("git");
        cmd.args(args);
        cmd.current_dir(&self.base);
        let out = cmd
            .output()
            .with_context(|| format!("Running git with {:?}", cmd.get_args()))?;
        if !out.status.success() {
            bail!(
                "Failure running git: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }

    pub fn init(&self) -> Result<()> {
        self._git(["init"]).map(|_| ())
    }

    pub fn add(&self, p: impl AsRef<Path>) -> Result<()> {
        let p = p.as_ref().as_os_str();
        self._git([OsStr::new("add"), p]).map(|_| ())
    }

    pub fn status(&self) -> Result<Vec<(char, char, PathBuf)>> {
        let all_stats = self._git(["status", "--porcelain=v1"])?;
        Ok(all_stats
            .lines()
            .map(|s| {
                let mut c = s.chars();
                let c1 = c.next().unwrap();
                let c2 = c.next().unwrap();
                c.next();
                let s = c.collect::<String>();
                (c1, c2, s.into())
            })
            .collect())
    }
}
