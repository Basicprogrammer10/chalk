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
    QueueableCommand,
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
    let lines = *args
        .get_one::<usize>("lines")
        .unwrap_or(&size().map(|x| x.1 as usize).unwrap_or(20));
    let mut page = *args.get_one::<usize>("start_page").unwrap_or(&0);

    // Get host
    let host = match misc::host_stuff(&args) {
        Some(i) => i,
        None => return,
    };

    let info = get_lines(&host, page, lines);

    if is_basic {
        basic(info);
        return;
    }

    let mut end = false;
    let mut line = 0;
    let mut loaded_lines = Vec::new();
    loaded_lines.extend(info.logs);
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

        if end && line + lines == loaded_lines.len() {
            queue!(
                stdout,
                Print("(START)".reversed()),
                MoveToColumn(0),
                MoveDown(1)
            )
            .unwrap();
        }

        for i in loaded_lines
            .iter()
            .skip(line)
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

        stdout.flush().unwrap();

        if let Event::Key(event) = read().unwrap() {
            match event.code {
                KeyCode::Up => line = line.saturating_sub(1),
                KeyCode::Down => {
                    line = line
                        .saturating_add(1)
                        .min(loaded_lines.len().saturating_sub(lines - 1))
                }
                KeyCode::Char('q') => break 'main,
                _ => {}
            }
        }

        if lines + line > (page + 1) * lines && !end {
            page += 1;
            let info = get_lines(&host, page, lines);
            end = end || info.logs.len() < lines;
            loaded_lines.extend(info.logs);
        }
    }

    execute!(stdout, Show, EnableLineWrap, LeaveAlternateScreen).unwrap();
}

fn get_lines(host: &str, page: usize, lines: usize) -> LogsInfo {
    let info = misc::deamon_req(
        "POST",
        &host,
        "logs",
        Some(json!({"page": page, "lines": lines})),
    )
    .unwrap();

    LogsInfo::deserialize(info).unwrap()
}

fn basic(info: LogsInfo) {
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

    if info.page == 0 {
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
