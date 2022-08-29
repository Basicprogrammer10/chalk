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
    server.route(Method::POST, "/logs", move |req| {
        let body = serde_json::from_str::<RequestData>(&req.body_string().unwrap()).unwrap();
        let logs = app.logs.read();
        let start = logs.len() <= (body.page + 1) * body.lines;

        let mut out = Vec::new();
        for i in logs
            .iter()
            .rev()
            .skip(body.page * body.lines)
            .take(body.lines)
            .rev()
        {
            out.push(json!({
                "type": i.log_type.to_string(),
                "text": i.data,
                "time": i.time
            }));
        }

        Response::new()
            .text(json!({ "logs": out, "start": start }))
            .content(Content::JSON)
    });
}
