use chrono::Utc;
use reqwest::Client;
use sqlx::{types::Uuid, PgPool};
use url::Url;

use crate::config::Config;

pub async fn fetch_from_odws(
    path: String,
    config: &Config,
    winnipeg_transit_api_address: String,
    maybe_incoming_message_id: Option<Uuid>,
    db: &PgPool,
) -> (reqwest::StatusCode, String) {
    let client = Client::new();
    let api_key = config.winnipeg_transit_api_key.clone();

    let base = Url::parse(&winnipeg_transit_api_address).unwrap();
    let mut url = base.join(&path).unwrap();

    url.query_pairs_mut().append_pair("api-key", &api_key);

    let api_response = client.get(url).send().await.unwrap();
    let status_code = api_response.status();
    let api_response_text = api_response.text().await.unwrap();

    let api_response_insertion_result = sqlx::query(
        r#"
        INSERT INTO api_responses (id, body, query, message_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(api_response_text.clone())
    .bind(path)
    .bind(maybe_incoming_message_id)
    .bind(Utc::now().naive_utc())
    .bind(Utc::now().naive_utc())
    .execute(db)
    .await;

    if let Err(e) = api_response_insertion_result {
        log::error!("Failed to insert API response: {}", e);
    }

    (status_code, api_response_text)
}
