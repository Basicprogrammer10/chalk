use std::sync::{atomic::Ordering, Arc};

use afire::{Content, Method, Response, Server};
use serde_derive::Deserialize;
use serde_json::{json, Value};

use crate::{
    misc::{self, ValadateType},
    App, Project,
};

#[derive(Deserialize)]
struct RequestData {
    token: String,
    name: String,
}

pub fn attach(server: &mut Server, app: Arc<App>) {
    server.route(Method::GET, "/app/info", move |req| {
        let body = serde_json::from_str::<RequestData>(&req.body_string().unwrap()).unwrap();
        if !ValadateType::Scoped(body.name.to_owned()).valadate(app.clone(), body.token) {
            return misc::error_res("Invalid Token");
        }

        let projects = app.projects.read();
        let app = match projects.iter().find(|x| x.name == body.name) {
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
                "status": *app.status.read(),
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
        "pid": pid,
        "memory": mem_info.size,
        "threads": stats.num_threads,
        "uptime": app.process.uptime.load(Ordering::Relaxed)
    }))
}
