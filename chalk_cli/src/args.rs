use std::str::FromStr;

use clap::{value_parser, Arg, ArgMatches, Command};

use crate::{commands::Commands, VERSION};

pub fn from_env() -> (Commands, ArgMatches) {
    let host = Arg::new("host")
        .takes_value(true)
        .short('d')
        .long("host")
        .help("The address of the daemon to connect to")
        .long_help("Defines the host +/ port of the daemon to connect to");
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
                .arg(&host),
            Command::new("logs")
                .about("Lets you view a daemons logs")
                .arg(host)
                .args([
                    Arg::new("basic")
                        .short('b')
                        .long("basic")
                        .help("Just prints the latest log entried to the terminal and exits"),
                    Arg::new("start_page")
                        .takes_value(true)
                        .value_parser(value_parser!(usize))
                        .short('p')
                        .long("page")
                        .help("The page to start from (line page * lines)"),
                    Arg::new("lines")
                        .takes_value(true)
                        .value_parser(value_parser!(usize))
                        .short('l')
                        .long("lines")
                        .help("Defines the number of lines to load"),
                ]),
        ])
        .get_matches();
    let m = m.subcommand().unwrap();

    (Commands::from_str(m.0).unwrap(), m.1.to_owned())
}
