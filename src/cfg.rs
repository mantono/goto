use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::fmt::color_choice;
use termcolor::ColorChoice;
use url::Url;

#[derive(Debug, StructOpt)]
#[structopt(name = "goto", about = "Web bookmarks utility")]
pub struct Config {
    /// Set verbosity level, 0 - 5
    ///
    /// Set the verbosity level, from 0 (least amount of output) to 5 (most verbose). Note that
    /// logging level configured via RUST_LOG overrides this setting.
    #[structopt(short, long = "verbosity", default_value = "1")]
    pub verbosity_level: u8,

    /// Print debug information
    ///
    /// Print debug information about current build for binary, useful for when an issue is
    /// encountered and reported
    #[structopt(short = "D", long = "debug")]
    pub print_dbg: bool,
    /// Set use of colors
    ///
    /// Enable or disable output with colors. By default, the application will
    /// try to figure out if colors are supported by the terminal in the current context, and use it
    /// if possible.
    /// Possible values are "on", "true", "off", "false", "auto".
    #[structopt(long = "colors", default_value = "auto")]
    colors: Flag,
    #[structopt(subcommand)]
    pub cmd: Command,
}

impl Config {
    pub fn colors(&self) -> ColorChoice {
        match self.colors {
            Flag::True => ColorChoice::Always,
            Flag::False => ColorChoice::Never,
            Flag::Auto => ColorChoice::Auto,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Flag {
    True,
    False,
    Auto,
}

impl FromStr for Flag {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "true" | "on" => Ok(Flag::True),
            "false" | "off" => Ok(Flag::False),
            "auto" => Ok(Flag::Auto),
            _ => Err(format!("Unrecognized option {}", s)),
        }
    }
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Add bookmark with URL
    ///
    /// Add bookmark with URL and optionally some tags
    Add { url: String, tags: Vec<String> },
    /// Open bookmark in browser
    ///
    /// Open a bookmark in the browser that is matching the given keywords. If several bookmarks
    /// match the keywords, they will be listed instead, with the option to select one bookmark and
    /// open it. If no bookmark is matching the keywords, the keywords will be directed to a
    /// search engine.
    Open { keywords: Vec<String> },
    /// List bookmarks
    ///
    /// List bookmarks matching the keywords, but do not open any bookmark.
    List { keywords: Vec<String> },
    /// Edit a bookmark
    ///
    /// Edit a bookmark in your editor of choice
    Edit { path: PathBuf },
    /// Delete bookmark
    Delete { path: PathBuf },
}
