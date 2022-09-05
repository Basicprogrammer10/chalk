use clap::ArgMatches;
use colored::Colorize;
use serde_json::json;

use crate::misc;

pub fn run(args: ArgMatches) {
    let name = args.get_one::<String>("app").unwrap();
    let signal = args.get_one::<String>("signal");

    // Get host
    let (host, token) = match misc::host_stuff(&args) {
        Some(i) => i,
        None => return,
    };

    let res = misc::deamon_req(
        "POST",
        &host,
        "app/action",
        Some(json!({
            "name": name,
            "action": "Stop",
            "signal": signal,
            "token": token
        })),
    )
    .unwrap();

    if let Some(e) = res.get("error") {
        println!("{}", format!("Error: `{}`", e.as_str().unwrap()).red());
    }

    println!("{}", "Ok".green());
}
