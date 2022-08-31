use std::io::{stdout, Write};

use chrono::{offset::TimeZone, Local};
use clap::ArgMatches;
use colored::Colorize;
use crossterm::{
    cursor::{Hide, MoveDown, MoveTo, MoveToColumn, Show},
    event::{read, Event, KeyCode},
    execute, queue,
    style::Print,
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, DisableLineWrap, EnableLineWrap,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use serde::Deserialize;
use serde_derive::Deserialize;
use serde_json::json;

use crate::misc::{self, t};

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

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogType {
    Error,
    Info,
}

pub fn run(args: ArgMatches) {
    // Get args
    let is_basic = args.contains_id("basic");
    let lines = *args
        .get_one::<usize>("lines")
        .unwrap_or(&size().map(|x| x.1 as usize).unwrap_or(20));
    let mut page = *args.get_one::<usize>("start_page").unwrap_or(&0);

    // Get token
    let token = match misc::get_token(&args) {
        Some(i) => i,
        None => {
            println!("{}", "[-] No token defined!".red());
            return;
        }
    };

    // Get host
    let host = match misc::host_stuff(&args, &token) {
        Some(i) => i,
        None => return,
    };

    if is_basic {
        basic(get_lines(&host, &token, page, lines, false, None));
        return;
    }

    let mut loaded_lines = Vec::new();
    let info = get_lines(&host, &token, page, lines, false, None);
    let mut end = info.end;
    let mut line: usize = 0;
    loaded_lines.extend(info.logs);
    let end_time = loaded_lines[0].time;

    let mut stdout = stdout();
    enable_raw_mode().unwrap();
    execute!(stdout, Hide, DisableLineWrap, EnterAlternateScreen).unwrap();

    'main: loop {
        let height = size().map(|x| x.1 as usize).unwrap_or(lines);
        queue!(stdout, MoveTo(0, 0), Clear(ClearType::All)).unwrap();

        if line == 0 {
            queue!(
                stdout,
                Print("(END)".reversed()),
                MoveToColumn(0),
                MoveDown(1)
            )
            .unwrap();
        }

        for i in
            loaded_lines
                .iter()
                .skip(line.saturating_sub(1))
                .take(t(line == 0, height - 1, height))
        {
            let time = Local.timestamp(i.time, 0);
            let line = format!("{} {}", time.format("[%Y-%m-%d] [%H:%M:%S]"), i.text);
            queue!(
                stdout,
                Print(i.log_type.colorize(&line)),
                MoveToColumn(0),
                MoveDown(1)
            )
            .unwrap();
        }

        if line + height > loaded_lines.len() + 1 {
            queue!(
                stdout,
                Print("(START)".reversed()),
                MoveToColumn(0),
                MoveDown(1)
            )
            .unwrap();
        }

        stdout.flush().unwrap();

        loop {
            if let Event::Key(event) = read().unwrap() {
                let old_line = line;
                match event.code {
                    KeyCode::Up => line = line.saturating_sub(1),
                    KeyCode::Down => line = line.saturating_add(1),
                    KeyCode::Char('q') => break 'main,
                    _ => {}
                }

                line = line.min(loaded_lines.len() - height + 2);
                if line != old_line {
                    break;
                }
            }
        }

        if lines + line > (page + 1) * lines && !end {
            page += 1;
            let info = get_lines(&host, &token, page, lines, false, Some(end_time));
            end = end || info.end;
            loaded_lines.extend(info.logs);
        }
    }

    execute!(stdout, Show, EnableLineWrap, LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
}

fn get_lines(
    host: &str,
    token: &str,
    page: usize,
    lines: usize,
    rev: bool,
    time: Option<i64>,
) -> LogsInfo {
    let info = misc::deamon_req(
        "POST",
        host,
        "logs",
        Some(json!({"page": page, "lines": lines, "end_time": time, "rev": rev, "token": token})),
    )
    .expect("Error getting data");

    LogsInfo::deserialize(info).expect("Invalid data fetched")
}

fn basic(info: LogsInfo) {
    if info.logs.is_empty() {
        println!("{}", "(EMPTY PAGE)".reversed());
        return;
    }

    if info.page == 0 {
        println!("{}", "(END)".reversed());
    }

    for i in info.logs {
        let time = Local.timestamp(i.time, 0);
        let line = format!("{} {}", time.format("[%Y-%m-%d] [%H:%M:%S]"), i.text);
        println!("{}", i.log_type.colorize(line));
    }

    if info.end {
        println!("{}", "(START)".reversed());
    }
}

impl LogType {
    fn colorize<T: AsRef<str>>(&self, msg: T) -> String {
        match self {
            LogType::Info => msg.as_ref().to_string(),
            LogType::Error => msg.as_ref().red().to_string(),
        }
    }
}
