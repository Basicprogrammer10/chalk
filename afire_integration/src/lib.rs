use std::env;

use afire::{Method, Request, Response, Server};
use ahash::{HashMap, HashMapExt};
use serde::Deserialize;

pub struct RemoteControl {
    // == Route Config ==
    method: Method,
    path: String,

    // == Systems ==
    systems: HashMap<String, Box<dyn Fn(String) -> String + Send + Sync>>,
    any: Vec<Box<dyn Fn(&Request, ControlData) + Send + Sync>>,

    // == Other ==
    enabled: bool,
    verification: String,
}

#[derive(Clone, Deserialize)]
pub struct ControlData {
    pub verification: String,
    pub action: String,
    pub data: String,
}

impl RemoteControl {
    pub fn new() -> Self {
        let key = env::var("CHALK-KEY");
        Self {
            method: Method::POST,
            path: "/control".to_owned(),

            systems: HashMap::new(),
            any: Vec::new(),

            enabled: key.is_ok(),
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

    /// (Request, ControlData)
    pub fn any(self, exe: impl Fn(&Request, ControlData) + Send + Sync + 'static) -> Self {
        let mut any = self.any;
        any.push(Box::new(exe));
        Self { any, ..self }
    }

    pub fn method(self, method: Method) -> Self {
        Self { method, ..self }
    }

    pub fn path<T: AsRef<str>>(self, path: T) -> Self {
        Self {
            path: path.as_ref().to_owned(),
            ..self
        }
    }

    /// For Debug Only
    pub fn enabled(self, enabled: bool) -> Self {
        println!("[-] Debug only option `enabled` was changed on RemoteControl");
        Self { enabled, ..self }
    }

    /// For Debug Only
    pub fn verification(self, verification: String) -> Self {
        println!("[-] Debug only option `verification` was changed on RemoteControl");
        Self {
            verification,
            ..self
        }
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

            self.any.iter().for_each(|x| x(&req, data.clone()));
            let out = executer(data.data.clone());
            Response::new().text(out)
        });
    }
}

fn err(error: &str) -> Response {
    Response::new().status(400).text(error)
}
