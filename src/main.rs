use warp::Filter;

#[tokio::main]
async fn main() {
    println!("Tau: request cannon spinning up...");

    let root = warp::path::end().map(|| {
        println!("Tau: served /");
        "Tau says: hello, world!\n"
    });

    // endpoint to check server responsiveness with minimal cost
    // should always return 200 OK with a content-length of 0
    let live = warp::path("live").and(warp::path::end()).map(|| {
        println!("Tau: served /live");
        ""
    });

    let routes = warp::get().and(root.or(live));

    warp::serve(routes).run(([127, 0, 0, 1], 1998)).await;
}
