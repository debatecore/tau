use axum::Router;
use tokio::net::TcpListener;
mod routes;
mod setup;

#[tokio::main]
async fn main() {
    let app = Router::new().merge(routes::routes());

    let addr = setup::get_socket_addr();
    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(e) => {
            panic!("error creating a listener ({e})");
        }
    };

    match axum::serve(listener, app).await {
        Ok(..) => (),
        Err(e) => {
            panic!("could not serve ({e})");
        }
    };
}
