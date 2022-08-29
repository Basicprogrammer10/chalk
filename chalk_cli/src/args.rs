use std::str::FromStr;

use clap::{Arg, ArgMatches, Command};

use crate::{commands::Commands, VERSION};

pub fn from_env() -> (Commands, ArgMatches) {
    let m = Command::new("chalk")
        .subcommand_required(true)
        .author("Connor Slade")
        .version(VERSION)
        .subcommands([
            Command::new("version")
                .about("Gets info on this cli program.")
                .arg(
                    Arg::new("basic")
                        .short('b')
                        .long("basic")
                        .help("Reduces the infomation printed"),
                ),
            Command::new("status")
                .about("Gets general infomation on the daemon")
                .args([Arg::new("host")
                    .takes_value(true)
                    .short('d')
                    .long("host")
                    .help("The address of the daemon to connect to")
                    .long_help("Defines the host +/ port of the daemon to connect to")]),
        ])
        .get_matches();
    let m = m.subcommand().unwrap();

    (Commands::from_str(m.0).unwrap(), m.1.to_owned())
}
