use std::env;

use afire::{Method, Response, Server};
use ahash::{HashMap, HashMapExt};
use serde::Deserialize;

type Executer = Box<dyn Fn(String) -> String + Send + Sync>;

pub struct RemoteControl {
    // == Route Config ==
    method: Method,
    path: String,

    // == Other ==
    enabled: bool,
    systems: HashMap<String, Executer>,
    verification: String,
}

#[derive(Deserialize)]
pub struct ControlData {
    verification: String,
    action: String,
    data: String,
}

impl RemoteControl {
    pub fn new() -> Self {
        let key = env::var("CHALK-KEY");
        Self {
            method: Method::POST,
            path: "/control".to_owned(),

            enabled: key.is_ok() || true,
            systems: HashMap::new(),
            verification: key.unwrap_or("".to_owned()),
        }
    }

    pub fn system(
        self,
        name: &str,
        exe: impl Fn(String) -> String + Send + Sync + 'static,
    ) -> Self {
        let mut systems = self.systems;
        systems.insert(name.to_owned(), Box::new(exe));

        Self { systems, ..self }
    }

    pub fn attach<App>(self, server: &mut Server<App>)
    where
        App: 'static + Send + Sync,
    {
        if !self.enabled {
            println!("[-] Chalk key not found. Disableing remote control.");
            return;
        }

        server.route(self.method, self.path, move |req| {
            let data =
                match serde_json::from_str::<ControlData>(&String::from_utf8_lossy(&req.body)) {
                    Ok(i) => i,
                    Err(_) => return err("Invalid Payload"),
                };

            if data.verification != self.verification {
                return err("Invalid Verification Token");
            }

            let executer = match self.systems.get(&data.action) {
                Some(i) => i,
                None => return err("Action not found"),
            };

            let out = executer(data.data);
            Response::new().text(out)
        });
    }
}

fn err(error: &str) -> Response {
    Response::new().status(400).text(error)
}
