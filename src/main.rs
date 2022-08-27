use std::sync::Arc;

mod api;
mod app;
mod config;
mod misc;
mod project;
use app::{App, LogType};
use misc::Timer;
use project::Project;

fn main() {
    let app = Arc::new(App::new());
    app.log(LogType::Info, "Starting");

    // Load Projects
    app.projects
        .write()
        .extend(Project::find_projects(app.clone()));

    // Start projects
    app.projects
        .read()
        .iter()
        .for_each(|x| x.start(app.clone()));

    // Start API
    api::start(app.clone());

    // Start an loop to poll tasks
    Timer::new(app.config.task_poll).start(|| {
        app.projects
            .read()
            .iter()
            .filter(|x| x.status.read().is_running())
            .for_each(Project::poll)
    });
}
