use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{extract::Path, response::IntoResponse, routing::get, Router};
use axum_extra::routing::SpaRouter;

#[tokio::main]
async fn main() {
    let spa = SpaRouter::new("/assets", "dist");

    let v1 = Router::new()
        .route("/property", get(property_list).post(property_create))
        .route(
            "/property/:id",
            get(property_read)
                .post(property_update)
                .delete(property_delete),
        );
    let api = Router::new().nest("/v1", v1);

    let app = Router::new().merge(spa).nest("/api", api);

    let socket_addr = SocketAddr::from((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080));
    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
        .expect("Unable to start backend");
}

async fn property_list() -> impl IntoResponse {
    "property list"
}

async fn property_update(Path(id): Path<u64>) -> impl IntoResponse {
    format!("update property {id}")
}

async fn property_read(Path(id): Path<u64>) -> impl IntoResponse {
    format!("read property {id}")
}

async fn property_create() -> impl IntoResponse {
    format!("new property")
}

async fn property_delete(Path(id): Path<u64>) -> impl IntoResponse {
    format!("delete property {id}")
}
