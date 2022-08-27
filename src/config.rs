use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub app_dir: String,
    pub socket_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_dir: "".to_owned(),
            socket_port: 3401,
        }
    }
}
