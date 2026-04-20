use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=.git/index");
    println!("cargo:rerun-if-changed=.git/HEAD");

    println!("cargo::rustc-env=GIT_COMMIT_HASH=UNKNOWN!");
    if let Ok(output) = Command::new("git").args(["rev-parse", "HEAD"]).output() {
        if let Ok(hash) = String::from_utf8(output.stdout) {
            if !hash.is_empty() && !hash.starts_with("fatal") {
                println!("cargo::rustc-env=GIT_COMMIT_HASH={}", hash);
            }
        }
    }

    println!("cargo::rustc-env=GIT_STATUS_PORCELAIN=UNKNOWN!");
    if let Ok(output) = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
    {
        if let Ok(status) = String::from_utf8(output.stdout) {
            if status.is_empty() {
                println!("cargo::rustc-env=GIT_STATUS_PORCELAIN=clean");
            } else {
                println!("cargo::rustc-env=GIT_STATUS_PORCELAIN=changed");
            }
        }
    }
}