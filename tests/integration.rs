use indoc::indoc;
use select::{document::Document, predicate::Name};
use speculoos::prelude::*;
use std::fs;
use textabus::{app, InjectableServices};
use tokio::net::TcpListener;
use wiremock::matchers::{any, method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn root_serves_placeholder() {
    let response = get(
        "/",
        InjectableServices {
            winnipeg_transit_api_address: None,
        },
    )
    .await
    .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(
        response.headers()["content-type"],
        "text/html; charset=utf-8"
    );

    let document = Document::from(response.text().await.unwrap().as_str());

    assert_that(&document.find(Name("h1")).next().unwrap().text()).contains("textabus");
}

#[tokio::test]
async fn twilio_serves_placeholder_with_unknown_body() {
    let response = get(
        "/twilio?Body=wha",
        InjectableServices {
            winnipeg_transit_api_address: None,
        },
    )
    .await
    .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(response.headers()["content-type"], "text/xml");

    let document = Document::from(response.text().await.unwrap().as_str());

    assert_that(&document.find(Name("body")).next().unwrap().text()).contains("textabus");
}

#[tokio::test]
async fn stop_number_returns_stop_name() {
    let mock_winnipeg_transit_api = MockServer::start().await;
    let mock_stop_schedule_response = fs::read_to_string("tests/fixtures/stop_schedule.json")
        .expect("Failed to read stop schedule fixture");

    Mock::given(method("GET"))
        .and(path_regex(r"^/v3/stops/.*/schedule.json$"))
        .respond_with(ResponseTemplate::new(200).set_body_string(mock_stop_schedule_response))
        .expect(1)
        .mount(&mock_winnipeg_transit_api)
        .await;

    let response = get(
        "/twilio?Body=10619",
        InjectableServices {
            winnipeg_transit_api_address: Some(mock_winnipeg_transit_api.uri()),
        },
    )
    .await
    .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(response.headers()["content-type"], "text/xml");

    let document = Document::from(response.text().await.unwrap().as_str());
    let body = &document.find(Name("body")).next().unwrap().text();

    assert_that(body).contains(indoc! {"
    10619 WB Graham@Vaughan (The Bay)
    12:18p 16 St Vital Ctr
    12:19p BLUE Downtown
    12:22p BLUE Downtown
    12:25p 60 UofM
    12:33p 18 Assin Park
    "});
}

async fn get(
    path: &str,
    mut services: InjectableServices,
) -> Result<reqwest::Response, reqwest::Error> {
    if services.winnipeg_transit_api_address.is_none() {
        let mock_winnipeg_transit_api = MockServer::start().await;

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(0)
            .named("Mock Winnipeg Transit API")
            .mount(&mock_winnipeg_transit_api)
            .await;

        services = InjectableServices {
            winnipeg_transit_api_address: Some("http://localhost:1313".to_string()),
        };
    }

    let app_address = spawn_app(services).await.address;

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
