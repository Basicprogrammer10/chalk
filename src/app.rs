use std::fs;
use std::process;

use colored::Colorize;
use directories::ProjectDirs;

use crate::config::Config;

pub struct App {
    pub app_dir: ProjectDirs,
    pub config: Config,
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

        Self { app_dir, config }
    }
}
