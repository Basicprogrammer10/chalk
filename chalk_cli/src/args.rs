use std::str::FromStr;

use clap::{Arg, ArgMatches, Command};

use crate::{commands::Commands, VERSION};

pub fn from_env() -> (Commands, ArgMatches) {
    let m = Command::new("chalk")
        .subcommand_required(true)
        .author("Connor Slade")
        .version(VERSION)
        .subcommand(
            Command::new("version")
                .about("Gets info on this cli program.")
                .arg(
                    Arg::new("basic")
                        .short('b')
                        .long("basic")
                        .help("Reduces the infomation printed"),
                ),
        )
        .get_matches();
    let m = m.subcommand().unwrap();

    (Commands::from_str(m.0).unwrap(), m.1.to_owned())
}
