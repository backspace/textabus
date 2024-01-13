mod helpers;

use helpers::get;

use select::{document::Document, predicate::Name};
use speculoos::prelude::*;
use sqlx::postgres::PgPool;
use textabus::InjectableServices;

#[sqlx::test]
async fn root_serves_placeholder(db: PgPool) {
    let response = get(
        "/",
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

    assert_that(&document.find(Name("h1")).next().unwrap().text()).contains("textabus");
}
