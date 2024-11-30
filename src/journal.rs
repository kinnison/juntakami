//! The journal itself

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

use eyre::{bail, Context, Result};
use time::{Date, OffsetDateTime};
use tracing::{info, warn};

use crate::{
    cli::InitArgs,
    config::{Configuration, CONFIG_FILENAME, JOURNAL_BASE, JOURNAL_ENTRY},
    filters::KeepDrop,
    git::Git,
    markdown::MarkdownFile,
};

pub struct NascentJournal {
    base: PathBuf,
}

pub struct Journal {
    base: PathBuf,
    git: Git,
    config: Configuration,
}
impl NascentJournal {
    /// Construct a Journal
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            base: path.as_ref().into(),
        }
    }

    /// Acquire the journal config
    pub fn load(self) -> Result<Journal> {
        let Self { base } = self;
        let git = Git::new(&base);
        let config = Configuration::load(base.join(CONFIG_FILENAME))?;
        Ok(Journal { base, git, config })
    }

    pub fn initialise(&self, args: &InitArgs) -> Result<()> {
        if std::fs::metadata(self.base.join(CONFIG_FILENAME)).is_ok() && !args.force() {
            bail!(
                "Unable to initialise. Journal already present at {}",
                self.base.display()
            );
        }
        info!("Initialising journal at {}", self.base.display());
        std::fs::create_dir_all(&self.base)
            .with_context(|| format!("Creating journal at: {}", self.base.display()))?;
        let config = Configuration::default();
        config.save(self.base.join(CONFIG_FILENAME))?;
        let git = Git::new(&self.base);
        git.init()?;
        git.add(CONFIG_FILENAME)?;
        Ok(())
    }
}

impl Journal {
    pub fn show_status(&self) -> Result<()> {
        let stats = self.git.status()?;
        for stat in stats {
            println!("{:?}", stat);
        }
        Ok(())
    }

    pub fn show_config(&self) -> Result<()> {
        let raw = self.config.raw_config()?;
        println!("{}", raw);
        Ok(())
    }

    fn log_filename(&self, date: Date) -> Result<PathBuf> {
        let leaf = date.format(self.config.log_pattern()).with_context(|| {
            format!(
                "Trying to format {date} using {:?}",
                self.config.log_pattern()
            )
        })?;
        Ok(self.base.join(leaf))
    }

    fn now() -> Result<OffsetDateTime> {
        time::OffsetDateTime::now_local().context("Trying to get today's date")
    }

    fn today() -> Result<Date> {
        Ok(Self::now()?.date())
    }

    pub fn load_recent(&self) -> Result<Option<MarkdownFile>> {
        // Start with tomorrow, so that we can find today if it already exists
        let Some(tomorrow) = Self::today()?.next_day() else {
            bail!("Unable to determine tomorrow's date")
        };

        let mut tries = 100; // Maximum of 100 days backwards to try
        let mut to_check = tomorrow;
        loop {
            tries -= 1;
            if tries == 0 {
                return Ok(None);
            }
            let log_entry = self.log_filename(to_check)?;
            if std::fs::exists(&log_entry).with_context(|| {
                format!("Attempting to detect existence of {}", log_entry.display())
            })? {
                break Some(MarkdownFile::load(log_entry)).transpose();
            } else {
                let Some(next_check) = to_check.previous_day() else {
                    bail!("Unable to compute day prior to {}", to_check);
                };
                to_check = next_check;
                continue;
            }
        }
    }

    pub fn prep(&self) -> Result<()> {
        let now = Self::now()?;
        let new_title = now
            .format(self.config.title())
            .context("Attempting to create new title")?;
        let new_created = now
            .format(self.config.created())
            .context("Attempting to create new created date")?;
        let mut loaded = self
            .load_recent()?
            .unwrap_or_else(MarkdownFile::new_log_entry);

        let new_filename = self.log_filename(now.date())?;
        if new_filename == loaded.origin() {
            warn!("Today's entry already exists, not changing it");
            return Ok(());
        }
        info!("Loaded {}", loaded.origin().display());

        loaded.set_title(&new_title);
        loaded.set_created(&new_created);
        loaded.set_author(self.config.author());

        loaded.filter_markdown(KeepDrop::new(loaded.keep_drop()), &self.config);

        std::fs::create_dir_all(new_filename.parent().unwrap()).with_context(|| {
            format!("Creating directories to lead to {}", new_filename.display())
        })?;
        info!("Writing {}", new_filename.display());
        loaded.write_raw(Some(new_filename))
    }

    pub fn edit(&self) -> Result<()> {
        let editor = self.config.editor();
        let mut cmd = Command::new(editor[0].as_ref());
        let log_filename = self.log_filename(Self::today()?)?;
        if !std::fs::exists(&log_filename)
            .with_context(|| format!("Checking for existence of {}", log_filename.display()))?
        {
            self.prep()?;
        }
        for arg in &editor[1..] {
            let arg = arg.as_ref();
            let arg = match arg {
                JOURNAL_BASE => self.base.as_os_str(),
                JOURNAL_ENTRY => log_filename.as_os_str(),
                arg => OsStr::new(arg),
            };
            cmd.arg(arg);
        }
        let mut res = cmd.spawn()?;
        let res = res.wait()?;
        if !res.success() {
            bail!("Editor failed to work? Exited {}", res.code().unwrap_or(-1));
        }
        Ok(())
    }
}
