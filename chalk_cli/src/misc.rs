use std::fs;
use std::process;

use clap::ArgMatches;
use colored::Colorize;
use directories::ProjectDirs;
use serde_json::{json, Value};
use ureq::Error;
use url::Url;

use crate::error::ActionError;

const TIME_UNITS: &[(&str, u16)] = &[
    ("second", 60),
    ("minute", 60),
    ("hour", 24),
    ("day", 30),
    ("month", 12),
    ("year", 0),
];

// == MISC ==

pub fn t<T>(case: bool, a: T, b: T) -> T {
    if case {
        return a;
    }

    b
}

pub fn tc<T, E>(case: bool, value: T, a: impl Fn(T) -> E, b: impl Fn(T) -> E) -> E {
    if case {
        return a(value);
    }

    b(value)
}

pub fn get_token(token: &ArgMatches, raw_host: String) -> Option<(String, bool)> {
    let config_dir = ProjectDirs::from("com", "connorcode", "chalk").unwrap();
    let token_storage_path = config_dir.data_dir().join("tokens.json");
    let token = token.get_one::<String>("token").cloned();

    if let Some(i) = token {
        return Some((i, true));
    }

    // Try reading cache?
    if let Ok(i) = fs::read_to_string(&token_storage_path) {
        let token_storage = serde_json::from_str::<Vec<[String; 2]>>(&i).unwrap();
        if let Some(i) = token_storage.iter().find(|x| x[0] == raw_host) {
            return Some((i[1].clone(), false));
        }
    }

    // Try reading config file
    let daemon_config = config_dir.preference_dir().join("config.toml");
    if daemon_config.exists() {
        let raw = fs::read_to_string(daemon_config).unwrap();
        let toml = toml::from_str::<toml::Value>(&raw).unwrap();
        return Some((toml.get("token")?.as_str()?.to_owned(), false));
    }

    // Fail
    None
}

pub fn deamon_req(
    method: &str,
    host: &str,
    path: &str,
    body: Option<Value>,
) -> Result<Value, ActionError>
where
{
    let req = ureq::request(method, &format!("{}{}", host, path));
    let req = match body {
        Some(i) => req.send_string(&i.to_string()),
        None => req.call(),
    };
    let req = match req {
        Ok(res) => res,
        Err(Error::Status(_, res)) => res,
        Err(e) => return Err(ActionError::Connect(Box::new(e))),
    };
    let data = req.into_string()?;
    let json = serde_json::from_str::<Value>(&data)?;

    if let Some(i) = json.get("error") {
        println!("{}", format!("[-] {}", i.as_str().unwrap()).red());
        process::exit(-1);
    }

    Ok(json)
}

pub fn format_elapsed(secs: u64) -> String {
    let mut secs = secs as f64;

    for i in TIME_UNITS {
        if i.1 == 0 || secs < i.1 as f64 {
            secs = secs.round();
            return format!("{} {}{}", secs, i.0, if secs == 1. { "" } else { "s" });
        }

        secs /= i.1 as f64;
    }

    format!("{} years", secs.round())
}

// == HOST STUFF ==

/// Returns (HOST, TOKEN)
pub fn host_stuff(args: &ArgMatches) -> Option<(String, String)> {
    // Get host
    let host = match parse_host(
        args.get_one::<String>("host")
            .map(|x| &x[..])
            .unwrap_or("http://localhost"),
    ) {
        Ok(i) => i,
        Err(e) => {
            println!("{} ({})", "[-] Invalid Host".red(), e);
            return None;
        }
    };

    let (token, new) = match get_token(args, host.clone()) {
        Some(i) => i,
        None => {
            println!("{}", "[-] No Token defined".red());
            return None;
        }
    };

    // Verify Host
    let req = deamon_req("GET", &host, "ping", Some(json!({ "token": token })));
    if let Err(i) = req {
        match i {
            ActionError::Read(e) => println!("{}\n{}", "[-] Error connecting to host".red(), e),
            ActionError::Parse(e) => println!("{}\n{}", "[-] Error reading from host".red(), e),
            ActionError::Connect(e) => println!("{}\n{}", "[-] Error Parsing host json".red(), e),
        };
        return None;
    }

    // If token is global, save it
    if new && req.unwrap().get("token").unwrap().as_str().unwrap() == "global" {
        println!("{}", "[*] Saveing token".yellow());
        save_token(&host, &token);
    }

    Some((host, token))
}

pub fn parse_host(inp: &str) -> Result<String, url::ParseError> {
    let url = Url::parse(inp)?;

    Ok(format!(
        "{}:{}{}",
        url.origin().unicode_serialization(),
        url.port().unwrap_or(3401),
        url.path()
    ))
}

fn save_token(host: &str, token: &str) {
    let config_dir = ProjectDirs::from("com", "connorcode", "chalk").unwrap();
    let token_storage_path = config_dir.data_dir().join("tokens.json");

    // If cache dosent exist make a new one
    if !token_storage_path.exists() {
        fs::create_dir_all(token_storage_path.parent().unwrap()).unwrap();
        fs::write(token_storage_path, json!([[host, token]]).to_string()).unwrap();
        return;
    }

    // If cache does exist read it, add new entry and resave
    let raw_token_storage = fs::read_to_string(&token_storage_path).unwrap();
    let mut token_storage = serde_json::from_str::<Vec<[String; 2]>>(&raw_token_storage).unwrap();
    token_storage.retain(|x| x[0] != host);
    token_storage.push([host.to_owned(), token.to_owned()]);

    fs::write(
        token_storage_path,
        serde_json::to_string(&token_storage).unwrap(),
    )
    .unwrap();
}
