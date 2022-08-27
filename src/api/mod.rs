use std::sync::Arc;
use std::thread;

use afire::Server;

use crate::App;

mod routes;

pub fn start(app: Arc<App>) {
    thread::Builder::new()
        .name("API".into())
        .spawn(|| {
            let mut server =
                Server::<Arc<App>>::new(&app.config.api_host, app.config.api_port).state(app);

            routes::attach(&mut server);

            server.start().unwrap();
        })
        .unwrap();
}
