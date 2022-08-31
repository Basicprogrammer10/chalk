use std::fmt::{self, Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use chrono::{Local, TimeZone, Utc};
use colored::Colorize;
use directories::ProjectDirs;
use parking_lot::RwLock;

use crate::config::Config;
use crate::Project;

pub struct App {
    // == App ==
    pub app_dir: ProjectDirs,
    pub config: Config,
    pub uptime: i64,

    // == Logs ==
    pub logs: RwLock<Vec<Log>>,
    pub last_log_save: AtomicU64,
    pub log_save_index: AtomicUsize,

    // == Projects ==
    pub projects: RwLock<Vec<Project>>,
    pub last_exit_try: AtomicU64,
}

pub struct Log {
    pub log_type: LogType,
    pub time: i64,
    pub data: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LogType {
    Error,
    Info,
}

impl App {
    pub fn new() -> Self {
        let app_dir = ProjectDirs::from("com", "connorcode", "chalk").unwrap();

        let config_path = app_dir.preference_dir().join("config.toml");
        let config = match fs::read_to_string(&config_path) {
            Ok(i) => toml::from_str(&i).unwrap(),
            Err(_) => {
                fs::create_dir_all(&config_path.parent().unwrap()).unwrap();
                fs::write(&config_path, toml::to_string(&Config::default()).unwrap()).unwrap();
                println!("{}", "[-] No config file found".red());
                println!(
                    "[*] Base config written to `{}`",
                    config_path.to_string_lossy()
                );
                process::exit(0);
            }
        };

        Self {
            app_dir,
            config,
            uptime: Utc::now().timestamp(),

            logs: RwLock::new(Vec::new()),
            last_log_save: AtomicU64::new(0),
            log_save_index: AtomicUsize::new(0),

            projects: RwLock::new(Vec::new()),
            last_exit_try: AtomicU64::new(0),
        }
    }

    pub fn log<T: AsRef<str>>(&self, log_type: LogType, text: T) {
        self.logs.write().push(Log {
            log_type,
            data: text.as_ref().to_string(),
            time: Utc::now().timestamp(),
        });

        // DEBUG!?
        if log_type == LogType::Error {
            println!("{}", text.as_ref().red());
            return;
        }

        println!("{}", text.as_ref());
    }

    pub fn log_tick(&self, force: bool) {
        let last_save_index = self.log_save_index.load(Ordering::Relaxed);
        let logs = self.logs.read();

        // Try save every minute
        if (self.last_log_save.load(Ordering::Relaxed) < 60 && !force)
            || last_save_index >= logs.len()
        {
            return;
        }

        println!("SAVEING LOGS");

        let log_folder = self.app_dir.preference_dir().join("logs");
        if !log_folder.exists() {
            fs::create_dir_all(&log_folder).unwrap();
        }

        let log_file = log_folder.join(Utc::now().format("%Y-%m-%d.log").to_string());
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .unwrap();

        let mut to_save = logs
            .iter()
            .skip(last_save_index as usize)
            .map(|x| x.to_string())
            .collect::<Vec<_>>();
        to_save.push("".to_owned());
        file.write_all(to_save.join("\n").as_bytes()).unwrap();

        self.log_save_index.store(logs.len(), Ordering::Relaxed);
        self.last_log_save
            .store(Utc::now().timestamp() as u64, Ordering::Relaxed);
    }
}

impl Display for LogType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(match self {
            LogType::Error => "error",
            LogType::Info => "info",
        })
    }
}

impl Display for Log {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "[{}] [{}] {}",
            Local.timestamp(self.time, 0).format("%H:%M:%S"),
            self.log_type,
            self.data
        ))
    }
}
