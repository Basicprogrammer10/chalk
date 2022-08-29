use std::fs;
use std::io::{Cursor, Read};
use std::str::FromStr;
use std::sync::Arc;

use afire::{Content, Method, Response, Server};
use flate2::read::GzDecoder;
use git2::{
    build::{CheckoutBuilder, RepoBuilder},
    Repository,
};
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
    // == Required ==
    name: String,
    action: ActionType,

    // == Stop action ==
    signal: Option<String>,

    // == Update action ==
    /// Binary data (BASE64(GZIP(RAW)))
    data: Option<String>,
    /// Git origin (EX: origin)
    remote: Option<String>,
    /// Git branch to pull from
    branch: Option<String>,
    /// Commit / Tag to checkout
    checkout: Option<String>,
    /// Should merging be forced
    force: Option<bool>,
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

                // TODO: username - token
                if let Some(i) = &project.config.git.repo {
                    let branch = body.branch.expect("No Branch defined");
                    let repo_path = project.project_path.join("repo");

                    let mut checkout_bld = CheckoutBuilder::new();
                    if body.force.unwrap_or(false) {
                        checkout_bld.force();
                    }

                    if !repo_path.exists() {
                        let mut repo = RepoBuilder::new();
                        repo.with_checkout(checkout_bld);
                        repo.clone(i, &repo_path).expect("Error cloneing repo");
                    }

                    let repo = Repository::open(repo_path).expect("Error opening repo");
                    let mut remote = repo
                        .find_remote(body.remote.as_deref().unwrap_or("origin"))
                        .expect("Remote not found");

                    let mut fo = git2::FetchOptions::new();
                    fo.download_tags(git2::AutotagOption::All);
                    remote.fetch(&[&branch], Some(&mut fo), None).unwrap();
                    let fetch_head = match body.checkout {
                        Some(i) => repo
                            .resolve_reference_from_short_name(&i)
                            .expect("Invalid refrence"),
                        None => repo.find_reference("FETCH_HEAD").unwrap(),
                    };
                    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head).unwrap();
                    if !misc::do_merge(&repo, &branch, fetch_commit).unwrap() {
                        return misc::error_res("Merge conflicts o.o");
                    }
                }

                if let Some(data) = body.data {
                    let base64_dec = base64::decode(data).expect("`data` is not valid base64");
                    let mut gzip_dec = GzDecoder::new(Cursor::new(base64_dec));

                    let mut out = Vec::new();
                    gzip_dec.read_to_end(&mut out).expect("Error decompressing");
                    fs::write(project.project_path.join("binary"), out)
                        .expect("Error writing new binary");
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