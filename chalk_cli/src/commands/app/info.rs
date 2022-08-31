use std::fmt::Display;

use clap::ArgMatches;
use colored::Colorize;
use serde::Deserialize;
use serde_derive::Deserialize;
use serde_json::json;

use crate::{
    commands::status,
    misc::{self, t, tc},
};

#[derive(Deserialize)]
struct InfoInfo {
    name: String,
    status: Status,
    info: Info,
    output: Output,
}

#[derive(Deserialize)]
struct Info {
    memory: u32,
    threads: u32,
}

#[derive(Deserialize)]
struct Output {
    stdout: String,
    stderr: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Running,
    Stoped,
    Crashed(bool, Option<i32>),
}

pub fn run(args: ArgMatches) {
    let name = args.get_one::<String>("app").unwrap();
    let host = match misc::host_stuff(&args) {
        Some(i) => i,
        None => return,
    };

    let raw = misc::deamon_req("POST", &host, "app/info", Some(json!({ "name": name })))
        .expect("Error getting data");
    let body = InfoInfo::deserialize(raw).expect("Invalid data fetched");
    let stdout = take_lines(body.output.stdout);
    let stderr = take_lines(body.output.stderr);

    // UI DESIGN (systemd inspired ofc)
    // ● PlasterBox
    //   Status: (Running, Stpoed, Crashed)
    //   Uptime: 100 hours
    //      Pid: 69
    //  Threads: 3
    //   Memory: 100mb
    //
    // == STDOUT ==
    // ----
    // ------
    // ---
    //
    // == STDERR ==
    // ------
    // ---
    // ----

    println!(
        "{} {}",
        body.status.dot().bold(),
        body.name.magenta().bold()
    );
    println!("  {} {}", "Status:".blue(), body.status);
    println!("  {} {}", "Uptime:".blue(), misc::format_elapsed(0));
    println!("     {} {}", "Pid:".blue(), "69");
    println!(" {} {}", "Threads:".blue(), body.info.threads);
    println!("  {} {}", "Memory:".blue(), body.info.memory);

    println!("\n{}", "== STDOUT ==".bold());
    println!(
        "{}{}",
        tc(stdout.0, (), |_| "(START)\n".reversed(), |_| "".white()),
        stdout.1
    );

    println!("\n{}", "== STDERR ==".bold());
    println!(
        "{}{}",
        tc(stderr.0, (), |_| "(START)\n".reversed(), |_| "".white()),
        stderr.1
    );
}

fn take_lines(inp: String) -> (bool, String) {
    let lines = inp.lines().collect::<Vec<_>>();

    (
        lines.len() <= 10 && !lines.is_empty(),
        lines
            .into_iter()
            .rev()
            .take(10)
            .rev()
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

impl Status {
    fn dot(&self) -> String {
        match self {
            Self::Running => "●".green(),
            Self::Stoped => "●".yellow(),
            Self::Crashed(_, _) => "●".red(),
        }
        .to_string()
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &match self {
                Self::Running => "Running".to_owned().green(),
                Self::Stoped => "Stoped".to_owned().yellow(),
                Self::Crashed(ok, status) => format!(
                    "Crashed{}",
                    tc(
                        status.is_some(),
                        (),
                        |_| format!(" ({})", status.unwrap()),
                        |_| "".to_owned()
                    )
                )
                .red(),
            }
            .to_string(),
        )
    }
}
