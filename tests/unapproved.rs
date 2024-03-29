mod helpers;

use helpers::{get, get_config};

use select::{document::Document, predicate::Name};
use serde_json::json;
use speculoos::prelude::*;
use sqlx::postgres::PgPool;
use textabus::{
    models::{Message, Number},
    routes::HELP_MESSAGE,
    InjectableServices,
};
use wiremock::{
    matchers::{body_string, method, path_regex},
    Mock, MockServer, ResponseTemplate,
};

#[sqlx::test]
async fn twilio_serves_welcome_to_and_registers_unknown_number_and_notifies_admin(db: PgPool) {
    let config = get_config();

    let mock_twilio: MockServer = MockServer::start().await;

    let twilio_create_message_body = serde_urlencoded::to_string([
        ("Body", &"New number: unknown".to_string()),
        ("To", &config.admin_number),
        ("From", &config.textabus_number),
    ])
    .expect("Could not encode message creation body");

    Mock::given(method("POST"))
        .and(path_regex(r"^/2010-04-01/Accounts/.*/Messages.json$"))
        .and(body_string(twilio_create_message_body.to_string()))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({})))
        .expect(1)
        .named("create message")
        .mount(&mock_twilio)
        .await;

    let response = get(
        "/twilio?Body=hey&From=unknown&To=textabus&MessageSid=SM1312",
        InjectableServices {
            db: db.clone(),
            twilio_address: Some(mock_twilio.uri()),
            winnipeg_transit_api_address: None,
        },
    )
    .await
    .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(response.headers()["content-type"], "text/xml");

    let document = Document::from(response.text().await.unwrap().as_str());

    assert_that(&document.find(Name("body")).next().unwrap().text()).contains("welcome to textabus. we don’t recognise you, please contact a maintainer to join the alpha test.");

    let [incoming_message, admin_message, outgoing_message]: [Message; 3] =
        sqlx::query_as("SELECT * FROM messages ORDER BY created_at")
            .fetch_all(&db)
            .await
            .expect("Failed to fetch messages")
            .try_into()
            .expect("Expected exactly 3 messages");

    assert_eq!(admin_message.body, "New number: unknown");
    assert_eq!(admin_message.origin, config.textabus_number);
    assert_eq!(admin_message.destination, config.admin_number);
    assert_eq!(admin_message.initial_message_id, Some(incoming_message.id));

    assert_eq!(incoming_message.body, "hey");
    assert_that(&outgoing_message.body).contains("maintainer");

    let [number]: [Number; 1] = sqlx::query_as("SELECT * FROM numbers")
        .fetch_all(&db)
        .await
        .expect("Failed to fetch numbers")
        .try_into()
        .expect("Expected exactly 1 number");

    assert_eq!(number.number, "unknown");
    assert!(!number.approved);
    assert!(!number.admin);
}

#[sqlx::test(fixtures("numbers-unapproved"))]
async fn twilio_ignores_a_known_but_not_approved_number(db: PgPool) {
    let response = get(
        "/twilio?Body=hey&From=unapproved&To=textabus&MessageSid=SM1312",
        InjectableServices {
            db: db.clone(),
            twilio_address: None,
            winnipeg_transit_api_address: None,
        },
    )
    .await
    .expect("Failed to execute request");

    assert_eq!(response.status(), 404);

    let [incoming_message]: [Message; 1] =
        sqlx::query_as("SELECT * FROM messages ORDER BY created_at")
            .fetch_all(&db)
            .await
            .expect("Failed to fetch messages")
            .try_into()
            .expect("Expected exactly 1 message");

    assert_eq!(incoming_message.body, "hey");
}

#[sqlx::test(fixtures("numbers-approved"))]
async fn twilio_serves_placeholder_with_unknown_body_to_approved_number_and_stores_messages(
    db: PgPool,
) {
    let config = get_config();

    let response = get(
        "/twilio?Body=wha&From=approved&To=textabus&MessageSid=SM1312",
        InjectableServices {
            db: db.clone(),
            twilio_address: None,
            winnipeg_transit_api_address: None,
        },
    )
    .await
    .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(response.headers()["content-type"], "text/xml");

    let document = Document::from(response.text().await.unwrap().as_str());

    assert_that(&document.find(Name("body")).next().unwrap().text()).contains("textabus");

    let [incoming_message, outgoing_message]: [Message; 2] =
        sqlx::query_as("SELECT * FROM messages ORDER BY created_at")
            .fetch_all(&db)
            .await
            .expect("Failed to fetch messages")
            .try_into()
            .expect("Expected exactly 2 messages");

    assert_eq!(incoming_message.body, "wha");
    assert_eq!(incoming_message.message_sid, Some("SM1312".to_string()));
    assert_eq!(incoming_message.origin, "approved");
    assert_eq!(incoming_message.destination, "textabus");
    assert_eq!(incoming_message.initial_message_id, None);

    assert_that(&outgoing_message.body).contains(HELP_MESSAGE);
    assert_that(&outgoing_message.body).contains(&config.root_url);

    assert_eq!(outgoing_message.origin, "textabus");
    assert_eq!(outgoing_message.destination, "approved");
    assert_eq!(
        outgoing_message.initial_message_id,
        Some(incoming_message.id)
    );

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_responses")
        .fetch_one(&db)
        .await
        .expect("Failed to fetch count");

    assert_eq!(count, 0);
}
