use clap::ArgMatches;

mod app;
mod logs;
mod status;
mod system;
mod version;

pub fn run(command: Commands) {
    match command.command {
        CommandType::Version => version::run(command.args),
        CommandType::Status => status::run(command.args),
        CommandType::System => system::run(command.args),
        CommandType::Logs => logs::run(command.args),

        CommandType::AppInfo => app::info::run(command.args),
        CommandType::AppStart => app::start::run(command.args),
        CommandType::AppStop => app::stop::run(command.args),
    }
}

pub struct Commands {
    pub args: ArgMatches,
    pub command: CommandType,
}

pub enum CommandType {
    // == ROOT COMMANDS ==
    Version,
    Status,
    System,
    Logs,

    // == APP COMMANDS ==
    AppInfo,
    AppStart,
    AppStop,
}

impl Commands {
    pub fn new(args: (&str, &ArgMatches)) -> Self {
        let sub_cmd = args.0.to_ascii_lowercase();

        if sub_cmd == "app" {
            let sub_sub = args.1.subcommand().unwrap();
            let command_type = match sub_sub.0.to_ascii_lowercase().as_str() {
                "info" => CommandType::AppInfo,
                "start" => CommandType::AppStart,
                "stop" => CommandType::AppStop,
                _ => unreachable!(),
            };

            return Self {
                command: command_type,
                args: sub_sub.1.to_owned(),
            };
        }

        let command_type = match sub_cmd.as_str() {
            "version" => CommandType::Version,
            "status" => CommandType::Status,
            "system" => CommandType::System,
            "logs" => CommandType::Logs,
            _ => unreachable!(),
        };

        Self {
            command: command_type,
            args: args.1.to_owned(),
        }
    }
}
