use std::sync::Arc;

use afire::{Content, Method, Response, Server};
use serde_json::json;

use crate::{misc, App};

pub fn attach(server: &mut Server<Arc<App>>) {
    server.stateful_route(Method::GET, "/get/{app}", |app, req| {
        let app_name = req.path_param("app").unwrap();

        let projects = app.projects.read();
        let app = match projects.iter().find(|x| x.name == app_name) {
            Some(i) => i,
            None => return misc::error_res("Invalid App"),
        };

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
                }
            }))
            .content(Content::JSON)
    });
}
