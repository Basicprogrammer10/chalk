use clap::ArgMatches;
use colored::Colorize;

use crate::{GITHUB, VERSION};

pub fn run(args: ArgMatches) {
    if args.contains_id("basic") {
        println!("chalk_cli version {VERSION}");
        return;
    }

    // Basic info
    println!(
        "{} {}\n",
        "🖍️  chalk_cli".magenta().bold(),
        format!("v{VERSION}").blue().bold()
    );

    // Git info (commit, branch, dirty)
    println!("{}  {}", "Git:".bright_magenta().bold(), env!("GIT_INFO"));

    // Links
    println!("{} {}", "Repo:".bright_magenta().bold(), GITHUB);
}
