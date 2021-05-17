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
        cfg::Command::Add { url, tags } => cmd::add(&mut buffer, &dir, url, tags),
        cfg::Command::Open { keywords } => cmd::open(&mut buffer, &dir, keywords),
        cfg::Command::Select { limit, keywords } => Ok(()),
        cfg::Command::List { keywords } => Ok(()),
        cfg::Command::Edit { path } => Ok(()),
        cfg::Command::Delete { path } => Ok(()),
    }
}

#[derive(Debug)]
pub enum Error {
    NotExistingFile,
    Formatting,
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
