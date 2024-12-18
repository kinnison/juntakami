use std::io::ErrorKind;
use std::path::Path;

pub use subplotlib;
use subplotlib::prelude::*;

use subplotlib::steplibrary::datadir::Datadir;
use subplotlib::steplibrary::runcmd::Runcmd;

#[step]
#[context(Datadir)]
#[context(Runcmd)]
pub fn _jt_binary_on_path(context: &ScenarioContext, bin: &Path) {
    let h = context.with(
        |context: &Datadir| {
            context
                .create_dir_all("bin")
                .and_then(|_| context.canonicalise_filename("bin"))
        },
        false,
    )?;
    #[cfg(unix)]
    std::os::unix::fs::symlink(bin, h.join("jt"))?;
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(bin, j.join("jt.exe"))?;
    context.with_mut(
        |context: &mut Runcmd| {
            context.prepend_to_path(h);
            Ok(())
        },
        false,
    )?;
}

#[step]
pub fn given_unique_home_dir(context: &Datadir) {
    let hpath = context.base_path();
    let jpath = hpath.join("journal");
    if !std::fs::metadata(hpath)?.is_dir() {
        throw!("$HOME is not a directory");
    }
    if let Err(e) = std::fs::metadata(jpath) {
        if e.kind() != ErrorKind::NotFound {
            throw!("Error when looking for $HOME/journal");
        }
    } else {
        throw!("$HOME/journal already exists");
    }
}

#[step]
pub fn journal_exists_at(context: &Datadir, loc: &Path) {
    let hpath = context.base_path();

    let jpath = if let Ok(pfx) = loc.strip_prefix("~/") {
        hpath.join(pfx)
    } else {
        hpath.join(loc)
    };

    if !std::fs::metadata(jpath)?.is_dir() {
        throw!("selected journal is not a directory");
    }
}

#[macro_export]
macro_rules! jt_binary_on_path {
    ($bin:expr) => {
        #[$crate::subplotlib::prelude::step]
        #[context(subplotlib::steplibrary::runcmd::Datadir)]
        #[context(subplotlib::steplibrary::runcmd::Runcmd)]
        fn jt_binary_on_path(context: &ScenarioContext) {
            juntakami_steps::_jt_binary_on_path::call(context, std::path::Path::new($bin))?;
        }
    };
}
