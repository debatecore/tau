fn get_version_string() -> String {
    return format!(
        "{}{}",
        if cfg!(debug_assertions) { "dev" } else { "" },
        env!("CARGO_PKG_VERSION")
    );
}

#[tokio::main]
async fn main() {
    println!("Tau cannon spinning up!");
}
