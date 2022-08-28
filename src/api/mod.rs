use std::sync::Arc;
use std::thread;

use afire::{Content, Response, Server};
use serde_json::json;

use crate::App;

mod routes;

pub fn start(app: Arc<App>) {
    thread::Builder::new()
        .name("API".into())
        .spawn(|| _start(app))
        .unwrap();
}

fn _start(app: Arc<App>) {
    // Create Server
    let mut server = Server::<()>::new(&app.config.api.host, app.config.api.port);

    // Change error handler to use json
    server.error_handler(|_req, err| {
        Response::new()
            .status(500)
            .text(json!({ "error": err }))
            .content(Content::JSON)
    });

    // Add routes
    routes::attach(&mut server, app.clone());

    // Start API
    server.start_threaded(app.config.api.workers).unwrap();
}
