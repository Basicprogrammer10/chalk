use std::sync::Arc;
use std::time::UNIX_EPOCH;

use afire::{Content, Method, Response, Server};
use serde_json::json;

use crate::{App, VERSION};

pub fn attach(server: &mut Server<Arc<App>>) {
    server.stateful_route(Method::GET, "/status", |app, _req| {
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
                "status": i.status.read().json()
            }));
        }

        // Logs
        let mut logs = Vec::new();
        for i in app.logs.read().iter().take(20) {
            logs.push(json!({
                "type": i.log_type.to_string(),
                "text": i.data,
                "time": i.time.duration_since(UNIX_EPOCH).unwrap().as_secs()
            }));
        }

        Response::new()
            .text(json!({
                "version": VERSION,
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
