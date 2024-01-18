mod helpers;

use helpers::{get, get_with_auth};

use select::{
    document::Document,
    predicate::{Attr, Class, Descendant, Name, Predicate},
};
use speculoos::prelude::*;
use sqlx::postgres::PgPool;
use textabus::InjectableServices;

#[sqlx::test(fixtures("numbers-approved", "messages"))]
async fn admin_serves_message_history(db: PgPool) {
    let response = get_with_auth(
        "/admin/messages",
        InjectableServices {
            db: db.clone(),
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
        .find(Name("tr").and(Attr("data-test-unapproved", "")))
        .next()
        .unwrap();
    assert_that(&unapproved_row.text()).contains("unapproved");
    assert_that(&unapproved_row.text()).contains("an unapproved");

    let approved_row = document
        .find(Name("tr").and(Attr("data-test-approved", "")))
        .next()
        .unwrap();
    assert_that(&approved_row.text()).contains("approved");
    assert_that(&approved_row.text()).contains("an approved");
}

#[sqlx::test]
async fn admin_rejects_without_auth(db: PgPool) {
    let messages_response = get(
        "/admin/messages",
        InjectableServices {
            db: db.clone(),
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
            winnipeg_transit_api_address: None,
        },
    )
    .await
    .expect("Failed to execute request");

    assert_eq!(numbers_response.status(), 401);
}
