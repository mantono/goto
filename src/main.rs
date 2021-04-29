#[macro_use]
extern crate clap;

mod args;
mod cfg;
mod dbg;
mod fmt;
mod logger;

use crate::cfg::Config;
use crate::dbg::dbg_info;
use crate::fmt::write;
use crate::logger::setup_logging;
use std::process;
use termcolor::{Color, StandardStream};

fn main() {
    let cfg: Config = Config::from_args(args::args());
    setup_logging(cfg.verbosity_level);

    if cfg.print_dbg {
        println!("{}", dbg_info());
        process::exit(0);
    }

    let mut stdout = StandardStream::stdout(cfg.use_colors);
    write(&mut stdout, "Hello, world!\n", Some(Color::Red));
}
