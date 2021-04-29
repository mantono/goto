use crate::fmt::color_choice;
use clap::ArgMatches;
use termcolor::ColorChoice;

pub struct Config {
    pub verbosity_level: u8,
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
