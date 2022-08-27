use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

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

    // Start an loop to poll tasks
    loop {
        let start = Instant::now();

        // Poll all tasks
        projects
            .iter()
            .filter(|x| x.status.read().is_running())
            .for_each(Project::poll);

        thread::sleep(Duration::from_millis(
            app.config
                .task_poll
                .saturating_sub(start.elapsed().as_millis() as u32) as u64,
        ))
    }
}
