use std::process;
use std::sync::{atomic::Ordering, Arc};

mod api;
mod app;
mod config;
mod ctrlc;
mod misc;
mod project;
use app::{App, LogType};
use misc::Timer;
use project::{Project, ProjectStatus};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let app = Arc::new(App::new());
    app.log(LogType::Info, format!("Starting (v{})", VERSION));

    // Init SIG(INT|TERM|HUP) handler
    ctrlc::init(app.clone());

    // Load Projects
    app.projects
        .write()
        .extend(Project::find_projects(app.clone()));

    // Start projects
    app.projects.read().iter().for_each(Project::start);

    // Start API
    api::start(app.clone());

    // Start an loop to poll tasks
    Timer::new(app.config.task_poll).start(|| {
        app.projects
            .read()
            .iter()
            .filter(|x| x.status.read().is_running())
            .for_each(Project::poll);

        // TODO: Cleanup
        if app.last_exit_try.load(Ordering::Relaxed) == 0 {
            return;
        }

        if app
            .projects
            .read()
            .iter()
            .filter(|x| *x.status.read() == ProjectStatus::Running)
            .count()
            == 0
        {
            process::exit(0);
        }
    });
}
