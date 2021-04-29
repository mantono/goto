use clap::{App, Arg, ArgMatches};

pub fn args<'a>() -> ArgMatches<'a> {
    let verbosity = Arg::with_name("verbosity")
        .takes_value(true)
        .default_value("1")
        .validator(|n: String| {
            let range = 0u8..=5u8;
            let n: u8 = n.parse::<u8>().unwrap();
            if range.contains(&n) {
                Ok(())
            } else {
                Err("Invalid value".to_string())
            }
        })
        .short("v")
        .long("verbosity")
        .help("Set verbosity level, 0 - 5")
        .long_help("Set the verbosity level, from 0 (least amount of output) to 5 (most verbose). Note that logging level configured via RUST_LOG overrides this setting.");

    let colors = Arg::with_name("colors")
        .long("colors")
        .short("C")
        .takes_value(true)
        .default_value("auto")
        .possible_values(&["auto", "never", "always", "ansi"])
        .value_name("setting")
        .help("Enable/disable colors")
        .long_help("Enable or disable output with colors. By default colors will be used if the terminal seems to support colors.");

    let debug = Arg::with_name("debug")
        .takes_value(false)
        .short("D")
        .long("debug")
        .help("Print debug information")
        .long_help("Print debug information about current build for binary, useful for when an issue is encountered and reported");

    let args: ArgMatches = App::new(crate_name!())
        .about("DESCRIPTION")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(verbosity)
        .arg(colors)
        .arg(debug)
        .get_matches();

    args
}
