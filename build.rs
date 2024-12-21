use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rerun-if-changed=.git/index");
    println!("cargo:rerun-if-changed=.git/HEAD");

    println!("cargo::rustc-env=GIT_COMMIT_HASH=UNKNOWN!");
    match Command::new("git").args(&["rev-parse", "HEAD"]).output() {
        Ok(output) => match String::from_utf8(output.stdout) {
            Ok(hash) => {
                if hash.is_empty() || hash.starts_with("fatal") {
                    return;
                }
                println!("cargo::rustc-env=GIT_COMMIT_HASH={}", hash)
            }
            Err(_) => (),
        },
        Err(_) => (),
    };
    println!("cargo::rustc-env=GIT_STATUS_PORCELAIN=UNKNOWN!");
    match Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
    {
        Ok(output) => match String::from_utf8(output.stdout) {
            Ok(status) => {
                if status.is_empty() {
                    println!("cargo::rustc-env=GIT_STATUS_PORCELAIN=clean");
                } else {
                    println!("cargo::rustc-env=GIT_STATUS_PORCELAIN=changed")
                }
            }
            Err(_) => (),
        },
        Err(_) => (),
    }
}
