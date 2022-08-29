use clap::ArgMatches;
use colored::Colorize;
use serde_json::Value;
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
pub fn deamon_req(host: &str, path: &str) -> Result<Value, ActionError>
where
{
    let req = ureq::get(&format!("{}{}", host, path)).call()?;
    let data = req.into_string()?;
    let json = serde_json::from_str(&data)?;

    Ok(json)
}

pub fn format_elapsed(secs: u64) -> String {
    let mut secs = secs as f64;

    for i in TIME_UNITS {
        if i.1 == 0 || secs < i.1 as f64 {
            secs = secs.round();
            return format!("{} {}{}", secs, i.0, if secs > 1.0 { "s" } else { "" });
        }

        secs /= i.1 as f64;
    }

    format!("{} years", secs.round())
}

// == HOST STUFF ==

pub fn host_stuff(args: &ArgMatches) -> Option<String> {
    // Get host
    let host = match parse_host(
        &args
            .get_one::<String>("host")
            .map(|x| &x[..])
            .unwrap_or("http://localhost"),
    ) {
        Ok(i) => i,
        Err(e) => {
            println!("{} ({})", "[-] Invalid Host".red(), e);
            return None;
        }
    };

    // Verify Host
    if let Err(i) = deamon_req(&host, "/ping") {
        match i {
            ActionError::Read(e) => println!("{}\n{}", "[-] Error connecting to host".red(), e),
            ActionError::Parse(e) => println!("{}\n{}", "[-] Error reading from host".red(), e),
            ActionError::Connect(e) => println!("{}\n{}", "[-] Error Parsing host json".red(), e),
        };
        return None;
    }

    Some(host)
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
