use std::fmt::{self, Display, Formatter};
use std::fs;
use std::process;
use std::sync::atomic::AtomicU64;

use chrono::Utc;
use colored::Colorize;
use directories::ProjectDirs;
use parking_lot::RwLock;

use crate::config::Config;
use crate::Project;

pub struct App {
    pub app_dir: ProjectDirs,
    pub config: Config,
    pub uptime: i64,

    pub projects: RwLock<Vec<Project>>,
    pub logs: RwLock<Vec<Log>>,
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

            projects: RwLock::new(Vec::new()),
            logs: RwLock::new(Vec::new()),
            last_exit_try: AtomicU64::new(0),
        }
    }

    pub fn log<T: AsRef<str>>(&self, log_type: LogType, text: T) {
        self.logs.write().push(Log {
            log_type,
            data: text.as_ref().to_string(),
            time: Utc::now().timestamp(),
        });

        // DEBUG!
        if log_type == LogType::Error {
            println!("{}", text.as_ref().red());
            return;
        }

        println!("{}", text.as_ref());
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
