use std::net::SocketAddr;

use axum::{
    routing::{get, post},
    Router,
};

async fn echo(body: String) -> &'static str {
    println!("{body}");
    "ok"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", post(echo)).route("/", get(echo));
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
