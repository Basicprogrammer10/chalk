use std::sync::Arc;

use afire::Server;

use crate::App;

mod app;
mod ping;
mod status;

pub fn attach(server: &mut Server, app: Arc<App>) {
    app::attach(server, app.clone());
    ping::attach(server, app.clone());
    status::attach(server, app);
}
