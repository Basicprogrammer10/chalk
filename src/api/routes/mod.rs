use std::sync::Arc;

use afire::Server;

use crate::App;

mod app;
mod status;

pub fn attach(server: &mut Server<Arc<App>>) {
    app::attach(server);
    status::attach(server);
}
