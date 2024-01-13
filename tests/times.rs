mod helpers;

use helpers::get;

use indoc::indoc;
use select::{document::Document, predicate::Name};
use speculoos::prelude::*;
use sqlx::postgres::PgPool;
use std::fs;
use textabus::{
    models::{ApiResponse, Message},
    InjectableServices,
};
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[sqlx::test(fixtures("numbers-approved"))]
async fn stop_number_returns_stop_schedule(db: PgPool) {
    let mock_winnipeg_transit_api = MockServer::start().await;
    let mock_stop_schedule_response = fs::read_to_string("tests/fixtures/times/stop_schedule.json")
        .expect("Failed to read stop schedule fixture");

    Mock::given(method("GET"))
        .and(path_regex(r"^/v3/stops/.*/schedule.json$"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(mock_stop_schedule_response.clone()),
        )
        .expect(1)
        .mount(&mock_winnipeg_transit_api)
        .await;

    let response = get(
        "/twilio?Body=10619&From=approved&To=textabus&MessageSid=SM1849",
        InjectableServices {
            db: db.clone(),
            winnipeg_transit_api_address: Some(mock_winnipeg_transit_api.uri()),
        },
    )
    .await
    .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(response.headers()["content-type"], "text/xml");

    let document = Document::from(response.text().await.unwrap().as_str());
    let body = &document.find(Name("body")).next().unwrap().text();

    let expected_body = indoc! {"
        10619 WB Graham@Vaughan (The Bay)
        12:16p 16 St Vital Ctr (1min ahead)
        12:19p BLUE Downtown (8min delay)
        12:22p BLUE Downtown
        12:25p 60 UofM
        "};

    assert_that(body).contains(expected_body);

    let [incoming_message, outgoing_message]: [Message; 2] =
        sqlx::query_as("SELECT * FROM messages ORDER BY created_at")
            .fetch_all(&db)
            .await
            .expect("Failed to fetch messages")
            .try_into()
            .expect("Expected exactly 2 messages");

    assert_eq!(incoming_message.body, "10619");
    assert_eq!(incoming_message.origin, "approved");
    assert_eq!(incoming_message.destination, "textabus");
    assert_eq!(incoming_message.initial_message_id, None);

    assert_eq!(outgoing_message.body, expected_body,);
    assert_eq!(outgoing_message.origin, "textabus");
    assert_eq!(outgoing_message.destination, "approved");
    assert_eq!(
        outgoing_message.initial_message_id,
        Some(incoming_message.id)
    );

    let api_response: ApiResponse = sqlx::query_as("SELECT * FROM api_responses LIMIT 1")
        .fetch_one(&db)
        .await
        .expect("Failed to fetch API response");

    assert_eq!(api_response.message_id, incoming_message.id);
    assert_eq!(api_response.body, mock_stop_schedule_response);
    assert_eq!(
        api_response.query,
        "/v3/stops/10619/schedule.json?usage=short"
    );
}