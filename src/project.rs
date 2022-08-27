use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::{self, Child, ExitStatus, Stdio};
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{App, LogType};

#[derive(Debug)]
pub struct Project {
    // == Static Settings ==
    /// The app friendly name
    pub name: String,

    /// Api Token (for externial requests)
    pub api_token: String,

    /// Git repo info
    pub git_info: GitInfo,

    /// The path to the app folder
    ///
    /// ```
    /// [project_path]
    /// | config.toml
    /// | git-repo
    /// | | ...
    /// | binary
    /// ```
    pub project_path: PathBuf,

    // == Process Stuff ==
    /// Current status of the process (for cli / automations?)
    pub status: RwLock<ProjectStatus>,

    /// Process handle for polling status and such
    pub process: Mutex<Option<Child>>,

    /// Process stdout
    pub stdout: RwLock<Vec<u8>>,

    /// Process stderr
    pub stderr: RwLock<Vec<u8>>,

    /// Arguments to run process with
    pub run_arguments: Vec<String>,

    /// Enviroment varables to run process with
    pub run_enviroment_vars: Vec<(String, String)>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawProjectConfig {
    pub name: String,
    pub api_token: String,

    pub run_args: Vec<String>,
    pub run_evars: HashMap<String, String>,

    pub git_repo: String,
    pub git_username: Option<String>,
    pub git_token: Option<String>,
}

#[derive(Debug)]
pub struct GitInfo {
    pub repo: String,
    pub username: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProjectStatus {
    Running,
    Stoped,
    Crashed(ExitStatus),
}

impl Project {
    fn from_raw(raw: RawProjectConfig, path: PathBuf) -> Self {
        Self {
            name: raw.name,
            api_token: raw.api_token,
            git_info: GitInfo {
                repo: raw.git_repo,
                username: raw.git_username,
                token: raw.git_token,
            },
            project_path: path,
            status: RwLock::new(ProjectStatus::Stoped),
            process: Mutex::new(None),
            stdout: RwLock::new(Vec::new()),
            stderr: RwLock::new(Vec::new()),
            run_arguments: raw.run_args,
            run_enviroment_vars: raw.run_evars.into_iter().collect(),
        }
    }

    pub fn start(&self, app: Arc<App>) {
        let binary_path = self.project_path.join("binary");

        if self.process.lock().is_some() {
            app.log(
                LogType::Error,
                format!("Process already started `{}`", self.name),
            );
            return;
        }

        if !binary_path.exists() {
            app.log(LogType::Error, format!("No runable binary `{}`", self.name));
            return;
        }

        app.log(LogType::Info, format!("Starting `{}`", self.name));
        let child = process::Command::new(binary_path)
            .args(&self.run_arguments)
            .envs(self.run_enviroment_vars.iter().cloned())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        *self.process.lock() = Some(child);
        *self.status.write() = ProjectStatus::Running;
    }

    pub fn poll(&self) {
        let mut process = self.process.lock();
        if process.is_none() {
            return;
        }

        let process = process.as_mut().unwrap();

        // Set App Status
        if let Some(i) = process.try_wait().unwrap() {
            *self.status.write() = ProjectStatus::Crashed(i);
        }

        // println!("START");
        // self.stdout
        //     .write()
        //     .extend(process.stdout.as_mut().unwrap().bytes().map(|x| x.unwrap()));
        // println!("END");
    }

    pub fn find_projects(app: Arc<App>) -> Vec<Project> {
        let app_dir = app.app_dir.preference_dir().join(&app.config.app_dir);
        let mut out = Vec::new();

        // Make app dir if not eggists
        if !app_dir.exists() {
            app.log(LogType::Info, "Apps folder not found. Makeing one.");
            fs::create_dir_all(&app_dir).unwrap();
        }

        for i in fs::read_dir(app_dir)
            .unwrap()
            .map(|x| x.unwrap())
            .filter(|x| x.path().is_dir())
        {
            app.log(
                LogType::Info,
                format!("Loading app `{}`", i.file_name().to_string_lossy()),
            );

            // Read config
            let app_config = i.path().join("config.toml");
            if !app_config.exists() {
                app.log(LogType::Error, "App config file not found! (config.toml)");
                continue;
            }
            let raw_config = fs::read_to_string(app_config).unwrap();

            // Load config
            let config = match toml::from_str::<RawProjectConfig>(&raw_config) {
                Ok(i) => i,
                Err(e) => {
                    app.log(LogType::Error, format!("Invalid app config: {}", e));
                    continue;
                }
            };

            out.push(Self::from_raw(config, i.path()));
        }

        out
    }
}

impl ProjectStatus {
    pub fn is_running(&self) -> bool {
        *self == ProjectStatus::Running
    }

    pub fn json(&self) -> Value {
        let state = match self {
            ProjectStatus::Running => "running",
            ProjectStatus::Stoped => "stoped",
            ProjectStatus::Crashed(i) => {
                return json!({ "state": "crashed", "is_ok": i.success(), "code": i.code()});
            }
        };

        json!({ "state": state })
    }
}
