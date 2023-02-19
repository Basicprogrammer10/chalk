use std::{env, fmt::Display};

use afire::{Method, Request, Response, Server};
use ahash::{HashMap, HashMapExt};
use serde_json::Value;

type System = Box<dyn Fn(&Value) -> Value + Send + Sync>;
type Any = Box<dyn Fn(&Request, &Value) + Send + Sync>;

pub struct RemoteControl {
    // == Route Config ==
    method: Method,
    path: String,

    // == Systems ==
    systems: HashMap<String, System>,
    any: Vec<Any>,

    // == Other ==
    enabled: bool,
    verification: String,
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
            verification: key.unwrap_or_else(|_| "".to_owned()),
        }
    }

    pub fn system(self, name: &str, exe: impl Fn(&Value) -> Value + Send + Sync + 'static) -> Self {
        let mut systems = self.systems;
        systems.insert(name.to_owned(), Box::new(exe));

        Self { systems, ..self }
    }

    /// (Request, ControlData)
    pub fn any(self, exe: impl Fn(&Request, &Value) + Send + Sync + 'static) -> Self {
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
            println!("[-] Chalk key not found. Disabling remote control.");
            return;
        }

        server.route(self.method, self.path, move |req| {
            let data = match serde_json::from_str::<Value>(&String::from_utf8_lossy(&req.body)) {
                Ok(i) => i,
                Err(e) => return err(format!("Invalid JSON: {e}")),
            };

            let verification = match data.get("verification").and_then(|i| i.as_str()) {
                Some(i) => i,
                None => return err("Missing Verification Token"),
            };
            if verification != self.verification {
                return err("Invalid Verification Token");
            }

            let action = match data.get("action").and_then(|i| i.as_str()) {
                Some(i) => i,
                None => return err("Missing Action"),
            };
            let executer = match self.systems.get(action) {
                Some(i) => i,
                None => return err("Action not found"),
            };

            let data = data.get("data").unwrap_or(&Value::Null);
            self.any.iter().for_each(|x| x(req, data));
            let out = executer(data);
            Response::new().text(out)
        });
    }
}

fn err(error: impl Display) -> Response {
    Response::new().status(400).text(error)
}

impl Default for RemoteControl {
    fn default() -> Self {
        Self::new()
    }
}
