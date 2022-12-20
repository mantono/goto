mod bookmark;
mod cfg;
mod cmd;
mod dbg;
mod io;
mod logger;
#[cfg(feature = "migrate")]
mod migrate;
mod tag;

use crate::cfg::Config;
use crate::dbg::dbg_info;
use crate::logger::setup_logging;
use bookmark::FileError;
use clap::Parser;
use dialoguer::theme::Theme;
use std::io::Write;
use std::{path::PathBuf, process};

fn main() -> Result<(), Error> {
    let cfg: Config = Config::parse();
    setup_logging(cfg.verbosity_level);

    let mut streams = io::Streams::new();

    let dir: PathBuf = dir().expect("Unable to find data directory");

    if cfg.print_dbg {
        writeln!(streams.ui(), "{}", dbg_info())?;
        writeln!(streams.ui(), "Using data directory {:?}", &dir)?;
        drop(streams);
        process::exit(0);
    }

    log::debug!("Using data directory {:?}", &dir);
    let theme: Box<dyn Theme> = cfg.theme();

    match cfg.cmd.unwrap_or_default() {
        cmd::Command::Add { url, tags } => cmd::add(streams, &dir, url, tags, &*theme),
        cmd::Command::Open {
            min_score,
            keywords,
        } => cmd::open(streams, &dir, keywords, min_score),
        cmd::Command::Select {
            min_score,
            limit,
            keywords,
        } => cmd::select(streams, &dir, keywords, limit, min_score, &*theme),
        #[cfg(feature = "migrate")]
        cmd::Command::Migrate => migrate::migrate(streams, &dir),
    }
}

#[derive(Debug)]
pub enum Error {
    NotExistingFile,
    Formatting,
    Serialization,
    OpenUrl,
    Other,
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

impl From<serde_yaml::Error> for Error {
    fn from(e: serde_yaml::Error) -> Self {
        log::error!("{}", e.to_string());
        Self::Serialization
    }
}

impl From<FileError> for Error {
    fn from(e: FileError) -> Self {
        match e {
            FileError::NotFound | FileError::NotFile => Error::NotExistingFile,
            FileError::Deserialize | FileError::Serialize => Error::Serialization,
            _ => Error::Other,
        }
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
