use select::{document::Document, predicate::Name};
use speculoos::prelude::*;
use textabus::{app, InjectableServices};
use tokio::net::TcpListener;

#[tokio::test]
async fn root_serves_placeholder() {
    let response = get("/").await.expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(
        response.headers()["content-type"],
        "text/html; charset=utf-8"
    );

    let document = Document::from(response.text().await.unwrap().as_str());

    assert_that(&document.find(Name("h1")).next().unwrap().text()).contains("textabus");
}

#[tokio::test]
async fn twilio_serves_placeholder() {
    let response = get("/twilio").await.expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(response.headers()["content-type"], "text/xml");

    let document = Document::from(response.text().await.unwrap().as_str());

    assert_that(&document.find(Name("body")).next().unwrap().text()).contains("textabus");
}

async fn get(path: &str) -> Result<reqwest::Response, reqwest::Error> {
    let app_address = spawn_app(InjectableServices {}).await.address;

    let client = reqwest::Client::new();
    let url = format!("{}{}", app_address, path);

    client.get(&url).send().await
}

struct TestApp {
    pub address: String,
}

async fn spawn_app(services: InjectableServices) -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    tokio::spawn(async move {
        axum::serve(listener, app(services).await.into_make_service())
            .await
            .unwrap();
    });

    TestApp { address }
}
