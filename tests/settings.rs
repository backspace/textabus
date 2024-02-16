mod helpers;

use helpers::get;

use select::{document::Document, predicate::Name};
use speculoos::prelude::*;
use sqlx::postgres::PgPool;
use textabus::{
    models::{Message, Number},
    InjectableServices,
};

#[sqlx::test(fixtures("numbers-approved"))]
async fn settings_clock_toggles_off_twelve_hour_field(db: PgPool) {
    let response = get(
        "/twilio?Body=settings clock&From=approved&To=textabus&MessageSid=SM1849",
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
    let body = &document.find(Name("body")).next().unwrap().text();

    let expected_body = "times will now be in 24h format";

    assert_that(body).contains(expected_body);

    let [incoming_message, outgoing_message]: [Message; 2] =
        sqlx::query_as("SELECT * FROM messages ORDER BY created_at")
            .fetch_all(&db)
            .await
            .expect("Failed to fetch messages")
            .try_into()
            .expect("Expected exactly 2 messages");

    assert_eq!(incoming_message.body, "settings clock");
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

    let [number]: [Number; 1] = sqlx::query_as("SELECT * FROM numbers")
        .fetch_all(&db)
        .await
        .expect("Failed to fetch numbers")
        .try_into()
        .expect("Expected exactly 1 number");

    assert!(!number.twelve_hour);
}

#[sqlx::test(fixtures("numbers-approved", "numbers-24h"))]
async fn settings_clock_toggles_on_twelve_hour_field(db: PgPool) {
    let response = get(
        "/twilio?Body=settings clock&From=approved&To=textabus&MessageSid=SM1849",
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
    let body = &document.find(Name("body")).next().unwrap().text();

    let expected_body = "times will now be in 12h format";

    assert_that(body).contains(expected_body);

    let [incoming_message, outgoing_message]: [Message; 2] =
        sqlx::query_as("SELECT * FROM messages ORDER BY created_at")
            .fetch_all(&db)
            .await
            .expect("Failed to fetch messages")
            .try_into()
            .expect("Expected exactly 2 messages");

    assert_eq!(incoming_message.body, "settings clock");
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

    let [number]: [Number; 1] = sqlx::query_as("SELECT * FROM numbers")
        .fetch_all(&db)
        .await
        .expect("Failed to fetch numbers")
        .try_into()
        .expect("Expected exactly 1 number");

    assert!(number.twelve_hour);
}

#[sqlx::test(fixtures("numbers-approved"))]
async fn settings_clock_does_not_work_with_raw_interface(db: PgPool) {
    let response = get(
        "/raw?body=settings clock",
        InjectableServices {
            db: db.clone(),
            twilio_address: None,
            winnipeg_transit_api_address: None,
        },
    )
    .await
    .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(
        response.headers()["content-type"],
        "text/plain; charset=utf-8"
    );

    let body = response.text().await.unwrap();

    let expected_body = "Cannot change settings with this interface";

    assert_eq!(body, expected_body);

    let [incoming_message, outgoing_message]: [Message; 2] =
        sqlx::query_as("SELECT * FROM messages ORDER BY created_at")
            .fetch_all(&db)
            .await
            .expect("Failed to fetch messages")
            .try_into()
            .expect("Expected exactly 2 messages");

    assert_eq!(incoming_message.body, "settings clock");
    assert_that(&incoming_message.origin).contains("127.0.0.1");
    assert_eq!(incoming_message.destination, "repl");
    assert_eq!(incoming_message.initial_message_id, None);

    assert_eq!(outgoing_message.body, expected_body,);
    assert_eq!(outgoing_message.origin, "repl");
    assert_that(&outgoing_message.destination).contains("127.0.0.1");
    assert_eq!(
        outgoing_message.initial_message_id,
        Some(incoming_message.id)
    );
}
