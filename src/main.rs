use warp::Filter;

fn get_version_string() -> String {
    return format!("{}{}",
        if cfg!(debug_assertions) {"dev"} else {""},
        env!("CARGO_PKG_VERSION")
    );
}

#[tokio::main]
async fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    } pretty_env_logger::init();

    // endpoint to check server responsiveness with minimal cost
    // should always return 200 OK with a content-length of 0
    let root = warp::path::end().map(|| {
        log::info!("served /");
        return warp::reply();
    });

    let v = warp::path("v").and(warp::path::end()).map(|| {
        log::info!("served /v");
        return get_version_string();
    });
    let version = warp::path("version").and(warp::path::end()).map(|| {
        log::info!("served /version");
        return format!("Tau cannon version {}\n", get_version_string());
    });

    let cors = warp::cors()
        .allow_any_origin();

    let routes = warp::get().and(
        root.or(v).or(version)
    ).with(cors);

    log::info!("Response cannon spinning up...");
    warp::serve(routes).run(([127, 0, 0, 1], 1998)).await;
}
