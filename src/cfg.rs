use std::str::FromStr;

use crate::cmd;
use dialoguer::theme::{ColorfulTheme, SimpleTheme, Theme};

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
    pub cmd: cmd::Command,
}

impl Config {
    pub fn theme(&self) -> Box<dyn Theme> {
        match self.colors {
            Flag::True => Box::new(ColorfulTheme::default()),
            Flag::False => Box::new(SimpleTheme),
            Flag::Auto => Box::new(ColorfulTheme::default()),
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
