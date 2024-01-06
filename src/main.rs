use tokio::net::TcpListener;

use textabus::{app, InjectableServices};

#[tokio::main]
async fn main() {
    let listener_address = "0.0.0.0:1312";
    let listener = TcpListener::bind(listener_address)
        .await
        .expect("Failed to bind port 1312");

    println!("textabus listening on port 1312");

    axum::serve(
        listener,
        app(InjectableServices {
            winnipeg_transit_api_address: Some("https://api.winnipegtransit.com".to_string()),
        })
        .await
        .into_make_service(),
    )
    .await
    .unwrap();
}
