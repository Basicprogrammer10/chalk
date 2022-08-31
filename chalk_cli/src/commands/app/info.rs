use clap::ArgMatches;
use serde::Deserialize;
use serde_derive::Deserialize;
use serde_json::json;

use crate::misc;

#[derive(Deserialize)]
struct InfoInfo {
    name: String,
    status: Status,
    info: Info,
    output: Output,
}

#[derive(Deserialize)]
struct Info {
    memory: u32,
    threads: u32,
}

#[derive(Deserialize)]
struct Output {
    stdout: String,
    stderr: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Running,
    Stoped,
    Crashed(bool, Option<i32>),
}

pub fn run(args: ArgMatches) {
    let name = args.get_one::<String>("app").unwrap();
    let host = match misc::host_stuff(&args) {
        Some(i) => i,
        None => return,
    };

    let raw = misc::deamon_req("POST", &host, "app/info", Some(json!({ "name": name })))
        .expect("Error getting data");
    let body = InfoInfo::deserialize(raw).expect("Invalid data fetched");

    // UI DESIGN (systemd inspired ofc)
    // ● PlasterBox (v0.1.0)
    //   Status: (Running, Stpoed, Crashed)
    //   Uptime: 100 hours
    //      Pid: 69
    //  Threads: 3
    //   Memory: 100mb
    //
    // == STDOUT ==
    // ----
    // ------
    // ---
    //
    // == STDERR ==
    // ------
    // ---
    // ----
}
