use std::sync::Arc;

use afire::{Content, Method, Response, Server};
use serde_derive::Deserialize;
use serde_json::json;

use crate::App;

#[derive(Deserialize)]
struct RequestData {
    page: usize,
    lines: usize,
}

pub fn attach(server: &mut Server, app: Arc<App>) {
    server.route(Method::GET, "/logs", move |req| {
        let body = serde_json::from_str::<RequestData>(&req.body_string().unwrap()).unwrap();
        let end = app.logs.read().len() <= (body.page + 1) * body.lines;

        let mut logs = Vec::new();
        for i in app
            .logs
            .read()
            .iter()
            .skip(body.page * body.lines)
            .take(body.lines)
        {
            logs.push(json!({
                "type": i.log_type.to_string(),
                "text": i.data,
                "time": i.time
            }));
        }

        Response::new()
            .text(json!({ "logs": logs, "end": end }))
            .content(Content::JSON)
    });
}
