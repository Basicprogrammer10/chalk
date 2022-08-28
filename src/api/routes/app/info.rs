use std::sync::Arc;

use afire::{Content, Method, Response, Server};
use serde_json::{json, Value};

use crate::{misc, App};

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

        // Get info on the process
        let pid = app.process.process.lock().as_ref().unwrap().id() as i32;
        let mem_info = procinfo::pid::statm(pid).unwrap();
        let stats = procinfo::pid::stat(pid).unwrap();

        Response::new()
            .text(json!({
                "name": app.name,
                "status": app.status.read().json(),
                "memory": mem_info.size,
                "threads": stats.num_threads,
                "output": {
                    "stdout": stdout,
                    "stderr": stderr,
                }
            }))
            .content(Content::JSON)
    });
}
