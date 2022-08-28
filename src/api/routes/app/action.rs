use std::fs;
use std::io::{Cursor, Read};
use std::str::FromStr;
use std::sync::Arc;

use afire::{Content, Method, Response, Server};
use flate2::read::GzDecoder;
use git2::Repository;
use nix::sys::signal::Signal;
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
    signal: Option<String>,
    data: Option<String>,
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

                let mut sig = Signal::SIGINT;
                if let Some(i) = body.signal {
                    sig = Signal::from_str(&i).expect("Invalid signal type");
                }

                project.stop(sig);
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

                if let Some(data) = body.data {
                    let base64_dec = base64::decode(data).expect("`data` is not valid base64");
                    let mut gzip_dec = GzDecoder::new(Cursor::new(base64_dec));

                    let mut out = Vec::new();
                    gzip_dec.read_to_end(&mut out).expect("Error decompressing");
                    fs::write(project.project_path.join("binary"), out)
                        .expect("Error writing new binary");
                }

                if let Some(i) = &project.config.git.repo {
                    // TODO: username - token
                    Repository::clone(i, project.project_path.join("repo"))
                        .expect("Error cloneing repo");
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
                projects.push(
                    Project::load_project(path, app.clone()).expect("New project is invalid"),
                );
                drop(projects);
            }
        }

        Response::new()
            .text(json!({"status": "ok"}))
            .content(Content::JSON)
    });
}
