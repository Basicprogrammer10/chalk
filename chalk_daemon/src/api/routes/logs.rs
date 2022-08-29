use std::sync::Arc;

use afire::{Content, Method, Response, Server};
use serde_derive::Deserialize;
use serde_json::json;

use crate::App;

#[derive(Deserialize)]
struct RequestData {
    page: usize,
    lines: usize,
    end_time: Option<i64>,
    rev: Option<bool>,
}

pub fn attach(server: &mut Server, app: Arc<App>) {
    server.route(Method::POST, "/logs", move |req| {
        let body = serde_json::from_str::<RequestData>(&req.body_string().unwrap()).unwrap();
        let logs = app.logs.read();
        let filterd = logs
            .iter()
            .filter(|x| body.end_time.is_none() || x.time <= body.end_time.unwrap())
            .collect::<Vec<_>>();
        let end = filterd.len() <= (body.page + 1) * body.lines;

        let mut out = Vec::new();
        for i in filterd
            .iter()
            .rev()
            .skip(body.page * body.lines)
            .take(body.lines)
        {
            out.push(json!({
                "type": i.log_type.to_string(),
                "text": i.data,
                "time": i.time
            }));
        }

        if let Some(true) = body.rev {
            out.reverse();
        }

        Response::new()
            .text(json!({ "logs": out, "end": end }))
            .content(Content::JSON)
    });
}
