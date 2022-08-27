use std::sync::Arc;

mod app;
mod config;
mod project;
use app::{App, LogType};
use project::Project;

fn main() {
    let app = Arc::new(App::new());
    app.log(LogType::Info, "Starting");

    let projects = Project::find_projects(app.clone());
    projects.iter().for_each(|x| x.start(app.clone()));

    ::std::thread::park();
}
