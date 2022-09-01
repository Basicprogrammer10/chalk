use clap::ArgMatches;
use colored::Colorize;
use serde::Deserialize;
use serde_derive::Deserialize;
use serde_json::json;

use crate::misc;

#[derive(Deserialize)]
struct SystemInfo {
    disk: Disk,
    load: Load,
    memory: Memory,
    os: Os,
    processes: u32,
}

#[derive(Deserialize)]
struct Disk {
    free: usize,
    total: usize,
}

#[derive(Deserialize)]
struct Load {
    #[serde(rename = "1m")]
    one: f32,
    #[serde(rename = "5m")]
    five: f32,
    #[serde(rename = "15m")]
    fifteen: f32,
}

#[derive(Deserialize)]
struct Memory {
    free: usize,
    total: usize,
}

#[derive(Deserialize)]
struct Os {
    release: String,
    #[serde(rename = "type")]
    os_type: String,
}

pub fn run(args: ArgMatches) {
    // Get host
    let (host, token) = match misc::host_stuff(&args) {
        Some(i) => i,
        None => return,
    };

    // Get info from daemon
    let info = misc::deamon_req("GET", &host, "status", Some(json!({ "token": token }))).unwrap();
    let info = SystemInfo::deserialize(info.get("system").unwrap()).unwrap();

    // localhost:3401
    // Processes: 38
    //      Load: 0.73, 0.32, 0.34 (1m, 5m, 15m)
    //      Disk: 10G / 15G (33%)
    //    Memory: 2G / 8G (25%)
    //        Os: Linux - 4.4.0-19041-Microsoft

    println!("● {}", host.magenta().bold());
    println!(
        "  {} {}",
        "Processes:".blue(),
        info.processes.to_string().magenta()
    );
    println!(
        "     {} {}",
        "Memory:".blue(),
        format!(
            "{} / {}",
            misc::format_storage_unit(info.memory.total.saturating_sub(info.memory.free)),
            misc::format_storage_unit(info.memory.total)
        )
        .magenta()
    );
    println!(
        "       {} {}",
        "Load:".blue(),
        format!(
            "{}, {}, {} (1m, 5m, 15m)",
            info.load.one, info.load.five, info.load.fifteen
        )
        .magenta()
    );
    println!(
        "       {} {}",
        "Disk:".blue(),
        format!(
            "{} / {}",
            misc::format_storage_unit(info.disk.total.saturating_sub(info.disk.free)),
            misc::format_storage_unit(info.disk.total)
        )
        .magenta()
    );
    println!(
        "         {} {}",
        "Os:".blue(),
        format!("{} - {}", info.os.os_type, info.os.release).magenta()
    );
}
