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
use bookmark::Bookmark;
use dialoguer::Editor;
use dialoguer::Input;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::Write;
use std::{fmt::Display, hash::Hash, str::FromStr};
use std::{path::PathBuf, process};
use structopt::StructOpt;
use termcolor::{Color, StandardStream};

fn main() -> Result<(), Error> {
    let cfg: Config = Config::from_args();
    setup_logging(cfg.verbosity_level);

    if cfg.print_dbg {
        println!("{}", dbg_info());
        process::exit(0);
    }

    let dir: PathBuf = dir().expect("Unable to find data directory");
    log::info!("Using data directory {:?}", &dir);

    let prot_prefix = regex::Regex::new("^https?://").unwrap();

    let stream = std::io::stdout();
    let mut buffer = std::io::BufWriter::new(stream);

    match cfg.cmd {
        cfg::Command::Add { url, tags } => {
            let url: String = if prot_prefix.is_match(&url) {
                url
            } else {
                format!("https://{}", url)
            };
            let url = url::Url::parse(&url).unwrap();
            let default: String = tags.join(", ");
            let tags: String = Input::new()
                .with_prompt("Tags")
                .default(default)
                .interact_text()?;

            let tags: Vec<Tag> = Tag::new_vec(tags);
            let bkm = bookmark::Bookmark::new(url, tags).unwrap();

            let full_path = dir.join(bkm.rel_path());
            std::fs::create_dir_all(full_path.parent().unwrap())?;
            println!("path {:?}", full_path);

            let bkm: Bookmark = if full_path.exists() {
                match Bookmark::from_file(&full_path) {
                    Some(prior_bkm) => bkm.merge(prior_bkm),
                    None => bkm,
                }
            } else {
                bkm
            };

            std::fs::write(full_path, bkm.to_string())?;

            writeln!(buffer, "{}", bkm)?;
        }
        cfg::Command::Open { keywords } => {}
        cfg::Command::List { keywords } => {}
        cfg::Command::Edit { path } => {}
        cfg::Command::Delete { path } => {}
    }

    Ok(())
}

lazy_static! {
    static ref TERMINATOR: Regex = Regex::new(r"[,\s]+").unwrap();
    static ref DISCARD: Regex = Regex::new(r#"[,\s"\\]+"#).unwrap();
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
pub struct Tag(String);

impl Tag {
    pub fn new<T: Into<String>>(tag: T) -> Result<Tag, TagError> {
        let tag: String = Self::normalize(&tag.into());
        if tag.is_empty() {
            Err(TagError::Empty)
        } else {
            Ok(Tag(tag))
        }
    }

    fn normalize(input: &str) -> String {
        DISCARD
            .replace_all(input, "")
            .to_lowercase()
            .trim()
            .to_string()
    }

    pub fn new_vec<T: Into<String>>(tags: T) -> Vec<Tag> {
        TERMINATOR
            .split(&tags.into())
            .filter_map(|t| Tag::from_str(t).ok())
            .collect()
    }
}

#[derive(Debug)]
pub enum TagError {
    Empty,
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Tag {
    type Err = TagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Tag::new(s)
    }
}

impl Hash for Tag {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

#[derive(Debug)]
enum Error {
    NotExistingFile,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        log::error!("{}", e.to_string());
        Self::NotExistingFile
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
