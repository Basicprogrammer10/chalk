use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    // Misc
    pub app_dir: String,
    pub task_poll: u32,

    // Api Config
    pub api: Api,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Api {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_dir: "./apps".to_owned(),
            task_poll: 1000,

            api: Api {
                host: "localhost".to_owned(),
                port: 3401,
                workers: 10,
            },
        }
    }
}
