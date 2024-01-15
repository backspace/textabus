mod helpers;

use helpers::get;

use select::{document::Document, predicate::Name};
use speculoos::prelude::*;
use sqlx::postgres::PgPool;
use std::env;
use textabus::{
    config::{ConfigProvider, EnvVarProvider},
    models::{Message, Number},
    routes::HELP_MESSAGE,
    InjectableServices,
};

#[sqlx::test]
async fn twilio_serves_welcome_to_and_registers_unknown_number(db: PgPool) {
    let response = get(
        "/twilio?Body=hey&From=unknown&To=textabus&MessageSid=SM1312",
        InjectableServices {
            db: db.clone(),
            winnipeg_transit_api_address: None,
        },
    )
    .await
    .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(response.headers()["content-type"], "text/xml");

    let document = Document::from(response.text().await.unwrap().as_str());

    assert_that(&document.find(Name("body")).next().unwrap().text()).contains("welcome to textabus. we don’t recognise you, please contact a maintainer to join the alpha test.");

    let [incoming_message, outgoing_message]: [Message; 2] =
        sqlx::query_as("SELECT * FROM messages ORDER BY created_at")
            .fetch_all(&db)
            .await
            .expect("Failed to fetch messages")
            .try_into()
            .expect("Expected exactly 2 messages");

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
    let env_config_provider = EnvVarProvider::new(env::vars().collect());
    let config = &env_config_provider.get_config();

    let response = get(
        "/twilio?Body=wha&From=approved&To=textabus&MessageSid=SM1312",
        InjectableServices {
            db: db.clone(),
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
