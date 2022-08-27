use std::sync::Arc;

mod app;
mod config;
mod project;
use app::{App, LogType};

fn main() {
    let app = Arc::new(App::new());
    let projects = project::Project::find_projects(app.clone());
    dbg!(projects);
    app.log(LogType::Info, "Starting");

    println!("Hello, world!");
}
