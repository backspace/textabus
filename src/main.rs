use tokio::net::TcpListener;

use textabus::app;

#[tokio::main]
async fn main() {
    let listener_address = "0.0.0.0:1312";
    let listener = TcpListener::bind(listener_address)
        .await
        .expect("Failed to bind port 1312");

    println!("textabus listening on port 1312");

    axum::serve(listener, app().await.into_make_service())
        .await
        .unwrap();
}
