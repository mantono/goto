#[macro_use]
extern crate structopt;

mod bookmark;
mod cfg;
mod cmd;
mod dbg;
mod fmt;
mod logger;
mod tag;

use crate::cfg::Config;
use crate::dbg::dbg_info;
use crate::logger::setup_logging;
use std::io::Write;
use std::{path::PathBuf, process};
use structopt::StructOpt;

fn main() -> Result<(), Error> {
    let cfg: Config = Config::from_args();
    setup_logging(cfg.verbosity_level);

    let stream = std::io::stdout();
    let mut buffer = std::io::BufWriter::new(stream);

    if cfg.print_dbg {
        writeln!(buffer, "{}", dbg_info())?;
        process::exit(0);
    }

    let dir: PathBuf = dir().expect("Unable to find data directory");
    log::debug!("Using data directory {:?}", &dir);

    match cfg.cmd {
        cmd::Command::Add { url, tags } => cmd::add(&mut buffer, &dir, url, tags),
        cmd::Command::Open {
            min_score,
            keywords,
        } => cmd::open(&mut buffer, &dir, keywords, min_score),
        cmd::Command::Select {
            min_score,
            limit,
            keywords,
        } => cmd::select(&mut buffer, &dir, keywords, limit, min_score),
        cmd::Command::List {
            min_score,
            limit,
            keywords,
        } => cmd::list(&mut buffer, &dir, keywords, limit, min_score),
        cmd::Command::Edit { path } => Ok(()),
        cmd::Command::Delete { path } => Ok(()),
    }
}

#[derive(Debug)]
pub enum Error {
    NotExistingFile,
    Formatting,
    Serialization,
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
