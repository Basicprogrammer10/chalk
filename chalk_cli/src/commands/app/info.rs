use std::fmt::Display;

use chrono::Utc;
use clap::ArgMatches;
use colored::Colorize;
use serde::Deserialize;
use serde_derive::Deserialize;
use serde_json::json;

use crate::misc::{self, tc};

#[derive(Deserialize)]
struct InfoInfo {
    name: String,
    status: Status,
    info: Option<Info>,
    output: Output,
}

#[derive(Deserialize)]
struct Info {
    pid: usize,
    memory: u32,
    threads: u32,
    uptime: u64,
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
    Crashed(Option<i32>),
}

pub fn run(args: ArgMatches) {
    let name = args.get_one::<String>("app").unwrap();

    // Get host
    let (host, token) = match misc::host_stuff(&args) {
        Some(i) => i,
        None => return,
    };

    let now = Utc::now().timestamp() as u64;
    let raw = misc::deamon_req(
        "GET",
        &host,
        "app/info",
        Some(json!({ "name": name, "token": token })),
    )
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
    if let Some(i) = body.info {
        if i.uptime != 0 {
            println!(
                "  {} {}",
                "Uptime:".blue(),
                misc::format_elapsed(now.saturating_sub(i.uptime))
            );
        }
        println!("     {} {}", "Pid:".blue(), i.pid);
        println!(" {} {}", "Threads:".blue(), i.threads);
        println!("  {} {}", "Memory:".blue(), i.memory);
    }

    println!(
        "\n{}\n{}",
        tc(stdout.0, "== STDOUT ==".bold(), |x| x.reversed(), |x| x),
        stdout.1
    );

    println!(
        "\n{}\n{}",
        tc(stderr.0, "== STDERR ==".bold(), |x| x.reversed(), |x| x),
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
            Self::Crashed(_) => "●".red(),
        }
        .to_string()
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            Self::Running => "Running".to_owned().green(),
            Self::Stoped => "Stoped".to_owned().yellow(),
            Self::Crashed(status) => format!(
                "Crashed{}",
                tc(
                    status.is_some(),
                    (),
                    |_| format!(" ({})", status.unwrap()),
                    |_| "".to_owned()
                )
            )
            .red(),
        })
    }
}
