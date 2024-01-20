mod helpers;

use helpers::{get, get_config, get_with_auth, post_with_auth};

use select::{
    document::Document,
    predicate::{Attr, Class, Descendant, Name, Predicate},
};
use serde_json::json;
use speculoos::prelude::*;
use sqlx::postgres::PgPool;
use textabus::{models::Message, routes::get_composed_approval_message, InjectableServices};
use wiremock::{
    matchers::{body_string, method, path_regex},
    Mock, MockServer, ResponseTemplate,
};

#[sqlx::test(fixtures("numbers-approved", "messages"))]
async fn admin_serves_message_history(db: PgPool) {
    let response = get_with_auth(
        "/admin/messages",
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
        "text/html; charset=utf-8"
    );

    let document = Document::from(response.text().await.unwrap().as_str());

    let row_count = document
        .find(Name("section").and(Class("exchange")))
        .count();
    assert_eq!(row_count, 2);

    let newest_exchange = document
        .find(Name("section").and(Class("exchange")))
        .next()
        .unwrap();

    assert_eq!(newest_exchange.find(Class("message")).count(), 2);

    let newest_first_message = newest_exchange.find(Class("message")).next().unwrap();

    let newest_first_message_from = newest_first_message.find(Class("from")).next().unwrap();
    assert_eq!(newest_first_message_from.text(), "stranger");
    assert_eq!(newest_first_message_from.attr("title").unwrap(), "stranger");

    assert_that(&newest_first_message.text()).contains("hello");
    assert_eq!(
        newest_first_message.attr("data-id").unwrap(),
        "b206e675-9220-4d95-94a8-a3dc0737557b"
    );

    let oldest_exchange = document
        .find(Name("section").and(Class("exchange")))
        .last()
        .unwrap();

    let oldest_exchange_time = oldest_exchange.find(Name("time")).next().unwrap();

    assert_eq!(
        oldest_exchange_time.attr("datetime").unwrap(),
        "2019-01-01T00:00:00"
    );
    assert_that(&oldest_exchange_time.text()).contains("Jan 01, 2019 00:00");

    assert_eq!(oldest_exchange.find(Class("message")).count(), 3);

    let oldest_first_message = oldest_exchange.find(Class("message")).next().unwrap();

    let oldest_first_message_from = oldest_first_message.find(Class("from")).next().unwrap();
    assert_eq!(oldest_first_message_from.text(), "an approved");
    assert_eq!(oldest_first_message_from.attr("title").unwrap(), "approved");

    assert_that(
        &oldest_first_message
            .find(Class("body"))
            .next()
            .unwrap()
            .text(),
    )
    .contains("hello");

    let oldest_last_message = oldest_exchange.find(Class("message")).last().unwrap();

    assert_that(&oldest_last_message.text()).contains("?");
}

#[sqlx::test(fixtures("numbers-approved", "numbers-unapproved"))]
async fn admin_serves_number_listings(db: PgPool) {
    let response = get_with_auth(
        "/admin/numbers",
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
        "text/html; charset=utf-8"
    );

    let document = Document::from(response.text().await.unwrap().as_str());
    let row_count = document.find(Descendant(Name("tbody"), Name("tr"))).count();

    assert_eq!(row_count, 2);

    let unapproved_row = document
        .find(Name("tr").and(Attr("data-unapproved", "")))
        .next()
        .unwrap();
    assert_that(&unapproved_row.text()).contains("unapproved");
    assert_that(&unapproved_row.text()).contains("an unapproved");

    let approved_row = document
        .find(Name("tr").and(Attr("data-approved", "")))
        .next()
        .unwrap();
    assert_that(&approved_row.text()).contains("approved");
    assert_that(&approved_row.text()).contains("an approved");
}

#[sqlx::test(fixtures("numbers-unapproved"))]
async fn test_approve_unapproved_number(db: PgPool) {
    let config = get_config();

    let mock_twilio: MockServer = MockServer::start().await;

    let approval_body = get_composed_approval_message();

    let twilio_create_message_body = serde_urlencoded::to_string([
        ("Body", &approval_body),
        ("To", &"unapproved".to_string()),
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

    let response = post_with_auth(
        "/admin/numbers/unapproved/approve",
        "",
        InjectableServices {
            db: db.clone(),
            twilio_address: Some(mock_twilio.uri()),
            winnipeg_transit_api_address: None,
        },
    )
    .await
    .expect("Failed to execute request");

    assert!(response.status().is_success());

    let unapproved_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM numbers WHERE approved = FALSE")
            .fetch_one(&db)
            .await
            .expect("Failed to fetch unapproved count");

    assert_eq!(unapproved_count, 0);

    let [approval_message]: [Message; 1] =
        sqlx::query_as("SELECT * FROM messages ORDER BY created_at")
            .fetch_all(&db)
            .await
            .expect("Failed to fetch messages")
            .try_into()
            .expect("Expected exactly 1 message");

    assert_eq!(approval_message.body, approval_body);
    assert_eq!(approval_message.origin, config.textabus_number);
    assert_eq!(approval_message.destination, "unapproved");
    assert_eq!(approval_message.initial_message_id, None,);
}

#[sqlx::test(fixtures("numbers-approved"))]
async fn test_unapprove_approved_number(db: PgPool) {
    let response = post_with_auth(
        "/admin/numbers/approved/unapprove",
        "",
        InjectableServices {
            db: db.clone(),
            twilio_address: None,
            winnipeg_transit_api_address: None,
        },
    )
    .await
    .expect("Failed to execute request");

    assert!(response.status().is_success());

    let approved_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM numbers WHERE approved = TRUE")
            .fetch_one(&db)
            .await
            .expect("Failed to fetch approved count");

    assert_eq!(approved_count, 0);
}

#[sqlx::test]
async fn admin_rejects_without_auth(db: PgPool) {
    let messages_response = get(
        "/admin/messages",
        InjectableServices {
            db: db.clone(),
            twilio_address: None,
            winnipeg_transit_api_address: None,
        },
    )
    .await
    .expect("Failed to execute request");

    assert_eq!(messages_response.status(), 401);

    let numbers_response = get(
        "/admin/numbers",
        InjectableServices {
            db: db.clone(),
            twilio_address: None,
            winnipeg_transit_api_address: None,
        },
    )
    .await
    .expect("Failed to execute request");

    assert_eq!(numbers_response.status(), 401);
}
