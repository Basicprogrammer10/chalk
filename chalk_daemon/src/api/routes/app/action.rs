use std::fmt::Display;
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use afire::{Content, Method, Response, Server};
use flate2::read::GzDecoder;
use git2::{
    build::{CheckoutBuilder, RepoBuilder},
    Repository,
};
use git2::{Cred, CredentialType, FetchOptions, RemoteCallbacks};
use nix::sys::signal::Signal;
use serde_derive::Deserialize;
use serde_json::json;

use crate::app::LogType;
use crate::{
    misc::{self, ValadateType},
    project::{Project, ProjectStatus},
    App,
};

#[derive(Deserialize)]
struct RequestData {
    // == Required ==
    token: String,
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
        if !ValadateType::Scoped(body.name.to_owned()).valadate(app.clone(), body.token) {
            return misc::error_res("Invalid Token");
        }

        let projects = app.projects.read();
        let project = match projects.iter().find(|x| x.name == body.name) {
            Some(i) => i,
            None => return misc::error_res("Invalid App"),
        };
        let name = project.name.to_owned();

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

                if let Some(i) = &project.config.git.repo {
                    let branch = body.branch.expect("No Branch defined");
                    let repo_path = project.project_path.join("repo");

                    let mut checkout_bld = CheckoutBuilder::new();
                    if body.force.unwrap_or(false) {
                        checkout_bld.force();
                    }

                    if !repo_path.exists() {
                        RepoBuilder::new()
                            .with_checkout(checkout_bld)
                            .fetch_options(git_auth_callback(project))
                            .clone(i, &repo_path)
                            .expect("Error cloneing repo");
                    }

                    let repo = Repository::open(repo_path).expect("Error opening repo");
                    let mut remote = repo
                        .find_remote(body.remote.as_deref().unwrap_or("origin"))
                        .expect("Remote not found");

                    remote
                        .fetch(&[&branch], Some(&mut git_auth_callback(project)), None)
                        .unwrap();
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
                    Project::load_project(path, app.clone())
                        .expect("New project is invalid. Project unloaded."),
                );
                drop(projects);
            }
        }

        app.log(
            LogType::Info,
            format!(
                "[WEB] [{}] Triggerd `{}` on `{}`",
                misc::get_ip(&req),
                body.action,
                name
            ),
        );

        Response::new()
            .text(json!({"status": "ok"}))
            .content(Content::JSON)
    });
}

fn git_auth_callback(project: &Project) -> FetchOptions {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, allowed_types| {
        let username = project
            .config
            .git
            .username
            .as_ref()
            .expect("No project git username defined");

        let token = project.config.git.token.as_ref();
        let ssh_key = project.config.git.ssh_key_file.as_ref();

        if allowed_types.contains(CredentialType::SSH_MEMORY) {
            if let Some(i) = ssh_key {
                return Cred::ssh_key(
                    username_from_url.unwrap_or(username),
                    None,
                    Path::new(i),
                    None,
                );
            }

            panic!("Tried to use ssh auth, no private key defined");
        }

        if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
            if let Some(i) = token {
                return Cred::userpass_plaintext(username_from_url.unwrap_or(username), i);
            }

            panic!("Tried to use token auth, no token defined");
        }

        panic!("No valid git auth found")
    });

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(callbacks);
    fo.download_tags(git2::AutotagOption::All);
    fo
}

impl Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Reload => "reload",
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Update => "update",
        })
    }
}
