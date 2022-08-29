use std::sync::Arc;

use afire::{Content, Method, Response, Server};
use serde_json::json;

use crate::{App, VERSION};

pub fn attach(server: &mut Server, _app: Arc<App>) {
    server.route(Method::GET, "/ping", |_req| {
        Response::new()
            .text(json!({ "version": VERSION }))
            .content(Content::JSON)
    });
}
