use std::fs;
use std::io::{Cursor, Read};
use std::sync::Arc;

use afire::{Content, Method, Response, Server};
use flate2::read::GzDecoder;
use git2::Repository;
use serde_derive::Deserialize;
use serde_json::json;

use crate::{
    misc,
    project::{Project, ProjectStatus},
    App,
};

#[derive(Deserialize)]
struct RequestData {
    /// App Name
    name: String,

    // Action info
    action: ActionType,
    action_data: Option<String>,
}

#[derive(Deserialize)]
enum ActionType {
    Stop,
    Start,
    Update,
    Reload,
}

pub fn attach(server: &mut Server, app: Arc<App>) {
    server.route(Method::POST, "/app/action", move |req| {
        let body = serde_json::from_str::<RequestData>(&req.body_string().unwrap()).unwrap();

        let projects = app.projects.read();
        let project = match projects.iter().find(|x| x.name == body.name) {
            Some(i) => i,
            None => return misc::error_res("Invalid App"),
        };

        match body.action {
            ActionType::Stop => {
                if *project.status.read() == ProjectStatus::Stoped {
                    return misc::error_res("App Already Stoped");
                }
                project.stop();
            }
            ActionType::Start => {
                if *project.status.read() == ProjectStatus::Running {
                    return misc::error_res("App Already Running");
                }
                project.start();
            }
            ActionType::Update => {
                if *project.status.read() == ProjectStatus::Running {
                    return misc::error_res("App is still running");
                }

                let action_data = body.action_data.unwrap();
                if !action_data.is_empty() {
                    let base64_dec = base64::decode(action_data).unwrap();
                    let mut gzip_dec = GzDecoder::new(Cursor::new(base64_dec));

                    let mut out = Vec::new();
                    gzip_dec.read_to_end(&mut out).unwrap();
                    fs::write(project.project_path.join("binary"), out).unwrap();
                }

                if let Some(i) = &project.git_info {
                    Repository::clone(&i.repo, project.project_path.join("repo")).unwrap();
                }
            }
            ActionType::Reload => {
                if *project.status.read() == ProjectStatus::Running {
                    return misc::error_res("App is still running");
                }
                let path = project.project_path.to_owned();

                drop(projects);

                let mut projects = app.projects.write();
                projects.retain(|x| x.name != body.name);
                projects.push(Project::load_project(path, app.clone()).unwrap());
                drop(projects);
            }
        }

        Response::new()
            .text(json!({"status": "ok"}))
            .content(Content::JSON)
    });
}
