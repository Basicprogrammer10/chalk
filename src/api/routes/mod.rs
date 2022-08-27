use std::sync::Arc;

use afire::Server;

use crate::App;

mod status;

pub fn attach(server: &mut Server<Arc<App>>) {
    status::attach(server);
}
