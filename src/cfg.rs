use crate::fmt::color_choice;
use clap::ArgMatches;
use termcolor::ColorChoice;

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
    pub use_colors: ColorChoice,
}

impl Config {
    pub fn from_args(args: ArgMatches) -> Config {
        let verbosity_level: u8 = args.value_of("verbosity").unwrap().parse::<u8>().unwrap();
        let print_dbg: bool = args.is_present("debug");
        let use_colors: ColorChoice = color_choice(args.value_of("colors").unwrap());

        Config {
            verbosity_level,
            print_dbg,
            use_colors,
        }
    }
}

#[derive(Debug, StructOpt)]
enum Command {}
