use std::process::Command;

fn main() {
    println!("cargo:rustc-env=GIT_COMMIT_HASH={}", "");
    match Command::new("git").args(&["rev-parse", "HEAD"]).output() {
        Ok(output) => match String::from_utf8(output.stdout) {
            Ok(hash) => println!("cargo:rustc-env=GIT_COMMIT_HASH={}", hash),
            Err(_) => (),
        },
        Err(_) => (),
    };
}
