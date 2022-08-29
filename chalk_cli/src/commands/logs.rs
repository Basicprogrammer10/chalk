use chrono::{offset::TimeZone, Local};
use clap::ArgMatches;
use colored::Colorize;
use serde::Deserialize;
use serde_derive::Deserialize;
use serde_json::json;

use crate::misc;

#[derive(Deserialize)]
struct LogsInfo {
    #[serde(skip)]
    page: usize,
    logs: Vec<Log>,
    start: bool,
}

#[derive(Deserialize)]
struct Log {
    text: String,
    #[serde(rename = "type")]
    log_type: LogType,
    time: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogType {
    Error,
    Info,
}

pub fn run(args: ArgMatches) {
    // Get args
    let is_basic = args.contains_id("basic");
    let lines = *args.get_one::<usize>("lines").unwrap_or(&20);
    let page = *args.get_one::<usize>("start_page").unwrap_or(&0);

    // Get host
    let host = match misc::host_stuff(&args) {
        Some(i) => i,
        None => return,
    };

    if is_basic {
        basic(host, lines, page);
        return;
    }
}

fn basic(host: String, lines: usize, page: usize) {
    let info = misc::deamon_req(
        "POST",
        &host,
        "logs",
        Some(json!({"page": page, "lines": lines})),
    )
    .unwrap();
    let info = LogsInfo::deserialize(info).unwrap();

    if info.logs.is_empty() {
        println!("{}", "(EMPTY PAGE)".reversed());
        return;
    }

    if info.start {
        println!("{}", "(END)".reversed());
    }

    for i in info.logs {
        let time = Local.timestamp(i.time, 0);
        let line = format!("{} {}", time.format("[%Y-%m-%d] [%H:%M:%S]"), i.text);
        println!("{}", i.log_type.colorize(line));
    }

    if page == 0 {
        println!("{}", "(START)".reversed());
    }
}

impl LogType {
    fn colorize(&self, msg: String) -> String {
        match self {
            LogType::Info => msg,
            LogType::Error => msg.red().to_string(),
        }
    }
}
