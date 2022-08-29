use std::str::FromStr;

use clap::ArgMatches;

mod status;
mod version;

pub fn run(command: Commands, args: ArgMatches) {
    match command {
        Commands::Version => version::run(args),
        Commands::Status => status::run(args),
    }
}

pub enum Commands {
    Version,
    Status,
    // Ideas:
    // Sysinfo
    // Logs
    // App
}

impl FromStr for Commands {
    type Err = ();

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        Ok(match str.to_ascii_lowercase().as_str() {
            "version" => Commands::Version,
            "status" => Commands::Status,
            _ => return Err(()),
        })
    }
}
