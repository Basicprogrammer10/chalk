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
    end: bool,
}

#[derive(Deserialize)]
struct Log {
    text: String,
    #[serde(rename = "type")]
    log_type: LogType,
    time: i64,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LogType {
    Error,
    Info,
}

pub fn run(args: ArgMatches) {
    // Get args
    let basic = args.contains_id("basic");
    let lines = args
        .get_one::<String>("lines")
        .map(|x| x.parse::<usize>().unwrap())
        .unwrap_or(20);
    let page = args
        .get_one::<String>("start_page")
        .map(|x| x.parse::<usize>().unwrap())
        .unwrap_or(0);

    // Get host
    let host = match misc::host_stuff(&args) {
        Some(i) => i,
        None => return,
    };

    if basic {
        let info =
            misc::deamon_req(&host, "logs", Some(json!({"page": page, "lines": lines}))).unwrap();
        let info = LogsInfo::deserialize(info).unwrap();

        for i in info.logs {
            let time = Local.timestamp(i.time, 0);
            let line = format!("{} {}", time.format("[%Y-%m-%d] [%H:%M:%S]"), i.text);
            println!("{}", i.log_type.colorize(line));
        }
        return;
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
