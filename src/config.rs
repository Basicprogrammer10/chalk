use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub app_dir: String,
    pub task_poll: u32,
    pub socket_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_dir: "./apps".to_owned(),
            task_poll: 1000,
            socket_port: 3401,
        }
    }
}
