use std::sync::Arc;

use afire::Server;

use crate::App;

mod action;
mod info;

pub fn attach(server: &mut Server, app: Arc<App>) {
    action::attach(server, app.clone());
    info::attach(server, app);
}
