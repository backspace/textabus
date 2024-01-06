use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "textabus: more to come" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:1312").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
