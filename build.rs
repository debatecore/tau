use std::process::Command;

fn main() {
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
    println!("cargo:rerun-if-changed=migrations");
}
