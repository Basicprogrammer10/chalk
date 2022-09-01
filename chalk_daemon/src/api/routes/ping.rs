use std::sync::Arc;

use afire::{Content, Method, Response, Server};
use serde_derive::Deserialize;
use serde_json::json;

use crate::{
    misc::{self, ValadateType},
    App, VERSION,
};

#[derive(Deserialize)]
struct RequestData {
    token: String,
}

pub fn attach(server: &mut Server, app: Arc<App>) {
    server.route(Method::GET, "/ping", move |req| {
        let body = serde_json::from_str::<RequestData>(&req.body_string().unwrap()).unwrap();
        if !ValadateType::Any.valadate(app.clone(), body.token.clone()) {
            return misc::token_error(app.clone(), req, body.token);
        }
        let token_type = ValadateType::token_type(app.clone(), body.token);

        Response::new()
            .text(json!({ "version": VERSION, "token": token_type.to_string() }))
            .content(Content::JSON)
    });
}
