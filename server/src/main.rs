use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{response::IntoResponse, routing::get, Router};
use axum_extra::routing::SpaRouter;
#[tokio::main]
async fn main() {
    let spa = SpaRouter::new("/assets", "dist");
    let api = Router::new().route("/api/hello", get(hello));

    let app = Router::new().merge(spa).merge(api);

    let socket_addr = SocketAddr::from((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080));
    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
        .expect("Unable to start backend");
}

async fn hello() -> impl IntoResponse {
    "hello from server!"
}
