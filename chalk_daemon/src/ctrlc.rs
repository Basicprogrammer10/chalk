use nix::sys::signal::Signal;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{App, Arc, LogType};

pub fn init(app: Arc<App>) {
    ctrlc::set_handler(move || {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let old = app.last_exit_try.load(Ordering::Relaxed);
        app.last_exit_try.store(now, Ordering::Relaxed);

        if now - old > 5 {
            app.log(LogType::Info, "Shutting down");
            app.projects
                .read()
                .iter()
                .for_each(|x| x.stop(Signal::SIGINT));
            return;
        }

        app.log(LogType::Info, "Shutting down (FORCE)");
        app.projects
            .read()
            .iter()
            .for_each(|x| x.stop(Signal::SIGKILL));
    })
    .unwrap();
}
