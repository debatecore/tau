use warp::Filter;

fn get_version_string() -> String {
    return format!("{}{}",
        if cfg!(debug_assertions) {"dev"} else {""},
        env!("CARGO_PKG_VERSION")
    );
}

#[tokio::main]
async fn main() {

    let root = warp::path::end().map(|| {
        println!("Tau: served /");
        "Tau says: hello, world\n"
    });

    // endpoint to check server responsiveness with minimal cost
    // should always return 200 OK with a content-length of 0
    let live = warp::path("live").and(warp::path::end()).map(|| {
        println!("Tau: served /live");
        warp::reply() // doing this instead of an empty string eliminates the content-type header
    });

    let v = warp::path("v").and(warp::path::end()).map(|| {
        println!("Tau: served /v");
        return get_version_string();
    });
    let version = warp::path("version").and(warp::path::end()).map(|| {
        println!("Tau: served /version");
        return format!("Tau cannon version {}\n", get_version_string());
    });

    let routes = warp::get().and(
        root.or(live).or(v).or(version)
    );

    println!("Tau: response cannon spinning up...");
    warp::serve(routes).run(([127, 0, 0, 1], 1998)).await;
}
