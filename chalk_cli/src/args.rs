use clap::{value_parser, Arg, Command};

use crate::{commands::Commands, VERSION};

pub fn from_env() -> Commands {
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
                .arg(&host)
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
            // Command::new("load").about("Loads new projects").arg(host),
            Command::new("app")
                .about("Commands that interact with a daemons app")
                .subcommand_required(true)
                .subcommands([Command::new("info")
                    .about("Gets info on a app")
                    .arg(host)
                    .arg(Arg::new("app").required(true))]),
        ])
        .get_matches();

    Commands::new(m.subcommand().unwrap())
}
