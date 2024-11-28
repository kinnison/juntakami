//! The journal itself

use std::path::{Path, PathBuf};

use eyre::{bail, Context, Result};
use tracing::info;

use crate::{
    cli::InitArgs,
    config::{Configuration, CONFIG_FILENAME},
    git::Git,
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
}
