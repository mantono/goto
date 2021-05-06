#[macro_use]
extern crate structopt;

mod bookmark;
mod cfg;
mod dbg;
mod fmt;
mod logger;

use crate::cfg::Config;
use crate::dbg::dbg_info;
use crate::fmt::write;
use crate::logger::setup_logging;
use std::{path::PathBuf, process};
use structopt::StructOpt;
use termcolor::{Color, StandardStream};

fn main() {
    let cfg: Config = Config::from_args();
    setup_logging(cfg.verbosity_level);

    if cfg.print_dbg {
        println!("{}", dbg_info());
        process::exit(0);
    }

    let dir: PathBuf = dir().expect("Unable to find data directory");
    log::info!("Using data directory {:?}", &dir);

    let prot_prefix = regex::Regex::new("^https?://").unwrap();

    match cfg.cmd {
        cfg::Command::Add { url } => {
            let url: String = if prot_prefix.is_match(&url) {
                url
            } else {
                format!("https://{}", url)
            };
            let url = url::Url::parse(&url).unwrap();
            let bkm = bookmark::Bookmark::new(url, None).unwrap();
            println!("{:?}", bkm.rel_path());
        }
        cfg::Command::Open { keywords } => {}
        cfg::Command::List { keywords } => {}
        cfg::Command::Edit { path } => {}
        cfg::Command::Delete { path } => {}
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
