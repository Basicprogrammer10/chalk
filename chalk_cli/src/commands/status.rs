use std::fmt::{self, Display, Formatter};

use chrono::Utc;
use clap::ArgMatches;
use colored::Colorize;
use serde::Deserialize;
use serde_derive::Deserialize;
use serde_json::json;

use crate::misc::{self, t, tc};

#[derive(Deserialize)]
struct StatusInfo {
    version: String,
    uptime: u64,
    apps: Vec<Project>,
}

enum SystemStatus {
    Good,
    Degraded,
    Yikes,
}

#[derive(Deserialize)]
struct Project {
    name: String,
    status: ProjectState,
    is_ok: Option<bool>,
    code: Option<i32>,
}

#[derive(Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectState {
    Running,
    Stoped,
    Crashed(bool, Option<i32>),
}

pub fn run(args: ArgMatches) {
    // Get host
    let (host, token) = match misc::host_stuff(&args) {
        Some(i) => i,
        None => return,
    };

    // Get info from daemon
    let info = misc::deamon_req("GET", &host, "status", Some(json!({ "token": token }))).unwrap();
    let info = StatusInfo::deserialize(info).unwrap();

    // Extrapalate from data
    let now = Utc::now().timestamp() as u64;
    let running = app_count(&info.apps, ProjectState::Running);
    let stoped = app_count(&info.apps, ProjectState::Stoped);
    let status = SystemStatus::from(&info);
    let total = info.apps.len();

    // Display it fancaly (like systemctl)
    // ● localhost:3401 (v0.1.0)
    //   Status: (All good, Degraded, Yikes!)
    //    Since: {} (mins | hours | days | ...)
    //  Running: {}
    //   Stoped: {} [{}]
    //
    //  [┬] Projects ({})
    //   └ ...

    println!(
        "{} {} {}",
        status.dot().bold(),
        host.magenta().bold(),
        format!("(v{})", info.version).cyan().bold()
    );
    println!("  {} {}", "Status:".blue(), status);
    println!(
        "  {} {}",
        "Uptime:".blue(),
        misc::format_elapsed(now.saturating_sub(info.uptime)).magenta()
    );
    println!(" {} {}", "Running:".blue(), running.to_string().magenta());
    println!(
        "  {} {} {}",
        "Stoped:".blue(),
        stoped.to_string().magenta(),
        format!("[{}]", total - running - stoped).red()
    );

    println!("\n {}┬{}", "[".blue(), "] Projects".blue());
    for (i, e) in info.apps.iter().enumerate() {
        println!(
            "  {} {} {}",
            t(i + 1 == total, "└", "├"),
            e.status.colorize(&e.name),
            tc(
                e.code.is_some(),
                (),
                |_| tc(
                    e.is_ok.unwrap_or(false),
                    format!("({})", e.code.unwrap()),
                    |x| x.yellow(),
                    |x| x.red()
                )
                .to_string(),
                |_| "".to_string()
            )
        );
    }
}

fn app_count(apps: &[Project], state: ProjectState) -> usize {
    let state = state.id();
    apps.iter().filter(|x| x.status.id() == state).count()
}

impl ProjectState {
    fn id(&self) -> usize {
        match self {
            ProjectState::Running => 0,
            ProjectState::Stoped => 1,
            ProjectState::Crashed(_, _) => 2,
        }
    }

    fn colorize(&self, inp: &str) -> String {
        match self {
            Self::Running => inp.green(),
            Self::Stoped => inp.yellow(),
            Self::Crashed(_, _) => inp.red(),
        }
        .to_string()
    }
}

impl SystemStatus {
    fn dot(&self) -> String {
        match self {
            Self::Good => "●".green(),
            Self::Degraded => "●".yellow(),
            Self::Yikes => "●".red(),
        }
        .to_string()
    }
}

impl From<&StatusInfo> for SystemStatus {
    fn from(from: &StatusInfo) -> Self {
        match app_count(&from.apps, ProjectState::Crashed(false, None)) {
            0 => Self::Good,
            1 => Self::Degraded,
            _ => Self::Yikes,
        }
    }
}

impl Display for SystemStatus {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        fmt.write_str(
            match self {
                Self::Good => "good".green(),
                Self::Degraded => "degraded".yellow(),
                Self::Yikes => "yikes".red(),
            }
            .to_string()
            .as_str(),
        )
    }
}
