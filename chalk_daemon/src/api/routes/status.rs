use std::sync::Arc;

use afire::{Content, Method, Response, Server};
use serde_derive::Deserialize;
use serde_json::json;

use crate::{
    misc::{self, BodyString, ValidateType},
    App, VERSION,
};

#[derive(Deserialize)]
struct RequestData {
    token: String,
}

pub fn attach(server: &mut Server, app: Arc<App>) {
    server.route(Method::GET, "/status", move |req| {
        let body = serde_json::from_str::<RequestData>(&req.body_string()).unwrap();
        if !ValidateType::Global.validate(app.clone(), body.token) {
            return misc::error_res("Invalid Token");
        }

        // Statem Status
        let disk = sys_info::disk_info().expect("Error getting Disk info");
        let mem = sys_info::mem_info().expect("Error getting Memory info");
        let load = sys_info::loadavg().expect("Error getting Load history");
        let proc = sys_info::proc_total().expect("Error getting process count");
        let os = sys_info::os_type().expect("Error getting OS type");
        let os_rel = sys_info::os_release().expect("Error getting OS info");

        // App Status
        let mut apps = Vec::new();
        for i in app.projects.read().iter() {
            apps.push(json!({
                "name": i.name,
                "status": *i.status.read()
            }));
        }

        // Logs
        let mut logs = Vec::new();
        for i in app.logs.read().iter().take(20) {
            logs.push(json!({
                "type": i.log_type.to_string(),
                "text": i.data,
                "time": i.time
            }));
        }

        Response::new()
            .text(json!({
                "version": VERSION,
                "uptime": app.uptime,
                "system": {
                    "disk": {
                        "total": disk.total,
                        "free": disk.free
                    },
                    "memory": {
                        "total": mem.total,
                        "free": mem.free
                    },
                    "load": {
                        "1m": load.one,
                        "5m": load.five,
                        "15m": load.fifteen,
                    },
                    "os": {
                        "type": os,
                        "release": os_rel
                    },
                    "processes": proc
                },
                "apps": apps,
                "logs": logs
            }))
            .content(Content::JSON)
    });
}
