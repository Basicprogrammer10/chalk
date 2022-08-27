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

        app.process.lock().as_mut().unwrap().stdout.take().unwrap();

        Response::new()
            .text(json!({
                "name": app.name,
                "status": app.status.read().json(),
                "output": {
                    "stdout": String::from_utf8_lossy(& app.stdout.read()),
                    "stderr": "",
                }
            }))
            .content(Content::JSON)
    });
}
