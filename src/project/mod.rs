use std::fs;
use std::path::PathBuf;
use std::process::{self, Child, ChildStderr, ChildStdout, ExitStatus, Stdio};
use std::sync::Arc;

use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use nonblock::NonBlockingReader;
use parking_lot::{Mutex, RwLock};
use serde_json::{json, Value};

use crate::{App, LogType};

mod config;
use config::ProjectConfig;

type Reader<T> = Mutex<Option<NonBlockingReader<T>>>;

pub struct Project {
    // == Static Settings ==
    /// The app friendly name
    pub name: String,

    /// app config
    pub config: ProjectConfig,

    // /// Git repo info
    // pub git_info: Option<GitInfo>,
    //
    /// The path to the app folder
    ///
    /// ```
    /// [project_path]
    /// | config.toml
    /// | repo
    /// | | ...
    /// | binary
    /// ```
    pub project_path: PathBuf,

    // == Process Stuff ==
    /// Current status of the process (for cli / automations?)
    pub status: RwLock<ProjectStatus>,

    /// Lower level process stuff
    pub process: Process,

    // == MISC ==
    /// Refrence to app
    app: Arc<App>,
}

pub struct Process {
    /// Process handle for polling status and such
    pub process: Mutex<Option<Child>>,

    /// Stdout Reader
    pub stdout_reader: Reader<ChildStdout>,

    /// Process stdout
    pub stdout: RwLock<Vec<u8>>,

    /// Stderr Reader
    pub stderr_reader: Reader<ChildStderr>,

    /// Process stderr
    pub stderr: RwLock<Vec<u8>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProjectStatus {
    Running,
    Stoped,
    Crashed(ExitStatus),
}

impl Project {
    fn from_raw(raw: ProjectConfig, path: PathBuf, app: Arc<App>) -> Self {
        Self {
            name: raw.name.to_owned(),
            config: raw,
            project_path: path,
            status: RwLock::new(ProjectStatus::Stoped),
            process: Process::new(),
            app,
        }
    }

    pub fn start(&self) {
        let binary_path = self.project_path.join("binary");

        if *self.status.read() == ProjectStatus::Running {
            self.app.log(
                LogType::Error,
                format!("Process already started `{}`", self.name),
            );
            return;
        }

        if !binary_path.exists() {
            self.app
                .log(LogType::Error, format!("No runable binary `{}`", self.name));
            return;
        }

        self.app
            .log(LogType::Info, format!("Starting `{}`", self.name));
        let mut child = process::Command::new(binary_path)
            .current_dir(self.project_path.join("repo"))
            .args(&self.config.run.arguments)
            .envs(&self.config.run.enviroment_vars)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        *self.process.stdout_reader.lock() =
            Some(NonBlockingReader::from_fd(child.stdout.take().unwrap()).unwrap());
        *self.process.stderr_reader.lock() =
            Some(NonBlockingReader::from_fd(child.stderr.take().unwrap()).unwrap());
        *self.process.process.lock() = Some(child);
        *self.status.write() = ProjectStatus::Running;
    }

    pub fn stop(&self, sig: Signal) {
        let mut raw_process = self.process.process.lock();
        if raw_process.is_none() {
            return;
        }

        self.app.log(
            LogType::Info,
            format!("Stopping `{}` with `{}`", self.name, sig),
        );

        let process = raw_process.as_mut().unwrap();
        signal::kill(Pid::from_raw(process.id() as i32), sig).unwrap();
    }

    pub fn poll(&self) {
        let mut process = self.process.process.lock();
        if process.is_none() {
            return;
        }

        let process = process.as_mut().unwrap();

        // Set App Status
        match process.try_wait().unwrap() {
            Some(x) if x.success() => *self.status.write() = ProjectStatus::Stoped,
            Some(i) => *self.status.write() = ProjectStatus::Crashed(i),
            _ => {}
        };

        // Process stdout / stderr
        // This is nonblocking due to the `NonBlockingReader`
        self.process
            .stdout_reader
            .lock()
            .as_mut()
            .unwrap()
            .read_available(self.process.stdout.write().as_mut())
            .unwrap();

        self.process
            .stderr_reader
            .lock()
            .as_mut()
            .unwrap()
            .read_available(self.process.stderr.write().as_mut())
            .unwrap();
    }

    pub fn load_project(path: PathBuf, app: Arc<App>) -> Option<Project> {
        app.log(
            LogType::Info,
            format!(
                "Loading app `{}`",
                path.file_name().unwrap().to_string_lossy()
            ),
        );

        // Read config
        let app_config = path.join("config.toml");
        if !app_config.exists() {
            app.log(LogType::Error, "App config file not found! (config.toml)");
            return None;
        }
        let raw_config = fs::read_to_string(app_config).expect("Error reading config file");

        // Load config
        let config = match toml::from_str::<ProjectConfig>(&raw_config) {
            Ok(i) => i,
            Err(e) => {
                app.log(LogType::Error, format!("Invalid app config: {}", e));
                return None;
            }
        };

        Some(Self::from_raw(config, path, app))
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
            if let Some(i) = Self::load_project(i.path(), app.clone()) {
                out.push(i);
            }
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

impl Process {
    pub fn new() -> Self {
        Self {
            process: Mutex::new(None),
            stdout_reader: Mutex::new(None),
            stderr_reader: Mutex::new(None),
            stdout: RwLock::new(Vec::new()),
            stderr: RwLock::new(Vec::new()),
        }
    }
}
