mod bookmark;
mod cfg;
mod cmd;
mod dbg;
#[cfg(feature = "git2")]
mod git;
mod io;
mod logger;
mod tag;

use crate::cfg::Config;
use crate::dbg::dbg_info;
use crate::logger::setup_logging;
use clap::Parser;
use dialoguer::theme::Theme;
use std::io::Write;
use std::{path::PathBuf, process};

fn main() -> Result<(), Error> {
    let cfg: Config = Config::parse();
    setup_logging(cfg.verbosity_level);

    let stream = std::io::stdout();
    let mut buffer = std::io::BufWriter::new(stream);

    let dir: PathBuf = dir().expect("Unable to find data directory");

    if cfg.print_dbg {
        writeln!(buffer, "{}", dbg_info())?;
        writeln!(buffer, "Using data directory {:?}", &dir)?;
        process::exit(0);
    }

    log::debug!("Using data directory {:?}", &dir);
    let theme: Box<dyn Theme> = cfg.theme();

    match cfg.cmd {
        cmd::Command::Add { url, tags } => cmd::add(&mut buffer, &dir, url, tags, &*theme),
        cmd::Command::Open {
            min_score,
            keywords,
        } => cmd::open(&mut buffer, &dir, keywords, min_score),
        cmd::Command::Select {
            min_score,
            limit,
            keywords,
        } => cmd::select(&mut buffer, &dir, keywords, limit, min_score, &*theme),
        #[cfg(feature = "git2")]
        cmd::Command::Sync => {
            git::sync(&dir)?;
            Ok(())
        }
    }
}

#[derive(Debug)]
pub enum Error {
    NotExistingFile,
    Formatting,
    Serialization,
    Sync,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        log::error!("{}", e.to_string());
        Self::NotExistingFile
    }
}

impl From<std::fmt::Error> for Error {
    fn from(e: std::fmt::Error) -> Self {
        log::error!("{}", e.to_string());
        Self::Formatting
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        log::error!("{}", e.to_string());
        Self::Serialization
    }
}

#[cfg(feature = "git2")]
impl From<git2::Error> for Error {
    fn from(e: git2::Error) -> Self {
        log::error!("Unable to sync git repository: {}", e);
        Self::Sync
    }
}

fn dir() -> Option<PathBuf> {
    match dirs_next::data_dir() {
        Some(mut dir) => {
            dir.push("goto");
            Some(dir)
        }
        None => match dirs_next::home_dir() {
            Some(mut dir) => {
                dir.push(".goto");
                Some(dir)
            }
            None => None,
        },
    }
}
