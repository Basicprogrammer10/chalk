mod args;
mod commands;
mod error;
mod misc;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB: &str = "https://github.com/Basicprogrammer10/chalk";

fn main() {
    let command = args::from_env();
    commands::run(command);
}
