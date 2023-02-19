use std::process::Command;

fn main() {
    let commit_hash = quick_cmd("git", &["rev-parse", "HEAD"]);
    let branch = quick_cmd("git", &["branch", "--show-current"]);
    let dirty = quick_cmd("git", &["status", "--porcelain"]) != Some("".to_owned());

    if commit_hash.is_none() && branch.is_none() {
        return println!("cargo:rustc-env=GIT_INFO=GIT NOT FOUND",);
    }

    println!(
        "cargo:rustc-env=GIT_INFO={} {}{}",
        branch.unwrap_or_default(),
        &commit_hash.unwrap_or_default()[0..8],
        if dirty { "*" } else { "" }
    );
}

fn quick_cmd(cmd: &str, args: &[&str]) -> Option<String> {
    Some(
        String::from_utf8(Command::new(cmd).args(args).output().ok()?.stdout)
            .unwrap()
            .replace(['\n', '\r'], ""),
    )
}
