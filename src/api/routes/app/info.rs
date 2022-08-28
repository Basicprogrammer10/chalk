use std::sync::Arc;

use afire::{Content, Method, Response, Server};
use serde_json::{json, Value};

use crate::{misc, App, Project};

pub fn attach(server: &mut Server, app: Arc<App>) {
    server.route(Method::POST, "/app/info", move |req| {
        let body = serde_json::from_str::<Value>(&req.body_string().unwrap()).unwrap();
        let app_name = body.get("name").unwrap().as_str().unwrap();

        let projects = app.projects.read();
        let app = match projects.iter().find(|x| x.name == app_name) {
            Some(i) => i,
            None => return misc::error_res("Invalid App"),
        };

        // Get std(out|err)
        let stdout = app.process.stdout.read();
        let stdout = String::from_utf8_lossy(&stdout);
        let stderr = app.process.stderr.read();
        let stderr = String::from_utf8_lossy(&stderr);

        Response::new()
            .text(json!({
                "name": app.name,
                "status": app.status.read().json(),
                "output": {
                    "stdout": stdout,
                    "stderr": stderr,
                },
                "info": get_info(app)
            }))
            .content(Content::JSON)
    });
}

fn get_info(app: &Project) -> Option<Value> {
    let i = app.process.process.lock();
    let i = i.as_ref()?;

    // Get info on the process
    let pid = i.id() as i32;
    let mem_info = procinfo::pid::statm(pid).ok()?;
    let stats = procinfo::pid::stat(pid).ok()?;

    Some(json!({
        "memory": mem_info.size,
        "threads": stats.num_threads,
    }))
}
