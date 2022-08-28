use git2::Repository;
use std::thread;
use std::time::{Duration, Instant};

use afire::{Content, Response};
use serde_json::json;

pub struct Timer {
    /// MS per loop
    pub time: u32,

    /// Update Start
    pub start: Instant,
}

impl Timer {
    pub fn new(time: u32) -> Self {
        Self {
            time,
            start: Instant::now(),
        }
    }

    pub fn start(&mut self, fun: impl Fn()) {
        loop {
            self.start = Instant::now();
            fun();

            thread::sleep(Duration::from_millis(
                self.time
                    .saturating_sub(self.start.elapsed().as_millis() as u32) as u64,
            ))
        }
    }
}

// Git stuff
// Modified from https://github.com/rust-lang/git2-rs examples

pub fn do_merge<'a>(
    repo: &'a Repository,
    remote_branch: &str,
    fetch_commit: git2::AnnotatedCommit<'a>,
) -> Result<bool, git2::Error> {
    let analysis = repo.merge_analysis(&[&fetch_commit])?;

    if analysis.0.is_fast_forward() {
        let refname = format!("refs/heads/{}", remote_branch);
        match repo.find_reference(&refname) {
            Ok(mut r) => {
                let name = match r.name() {
                    Some(s) => s.to_string(),
                    None => String::from_utf8_lossy(r.name_bytes()).to_string(),
                };
                r.set_target(
                    fetch_commit.id(),
                    &format!(
                        "Fast-Forward: Setting {} to id: {}",
                        name,
                        fetch_commit.id()
                    ),
                )?;
                repo.set_head(&name)?;
                repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
            }
            Err(_) => {
                repo.reference(
                    &refname,
                    fetch_commit.id(),
                    true,
                    &format!("Setting {} to {}", remote_branch, fetch_commit.id()),
                )?;
                repo.set_head(&refname)?;
                repo.checkout_head(Some(
                    git2::build::CheckoutBuilder::default()
                        .allow_conflicts(true)
                        .conflict_style_merge(true)
                        .force(),
                ))?;
            }
        };

        return Ok(true);
    }

    if analysis.0.is_normal() {
        let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
        return normal_merge(&repo, &head_commit, &fetch_commit);
    }

    Ok(true)
}

fn normal_merge(
    repo: &Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
) -> Result<bool, git2::Error> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(false);
    }
    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;

    let msg = format!("Merge: {} into {}", remote.id(), local.id());
    let sig = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;

    repo.checkout_head(None)?;
    Ok(true)
}

// Misc Functions
pub fn error_res<T: AsRef<str>>(err: T) -> Response {
    Response::new()
        .status(400)
        .text(json!({"error": err.as_ref()}))
        .content(Content::JSON)
}
