mod helpers;

use helpers::get;

use assertables::assert_starts_with;
use indoc::indoc;
use select::{document::Document, predicate::Name};
use speculoos::prelude::*;
use sqlx::postgres::PgPool;
use std::fs;
use textabus::{
    models::{ApiResponse, Message},
    InjectableServices,
};
use wiremock::matchers::{method, path, path_regex, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[sqlx::test(fixtures("numbers-approved"))]
async fn stops_returns_stops_and_routes_near_a_location(db: PgPool) {
    let mock_winnipeg_transit_api: MockServer = MockServer::start().await;

    let mock_locations_response = fs::read_to_string("tests/fixtures/stops/locations.json")
        .expect("Failed to read locations fixture");

    Mock::given(method("GET"))
        .and(path_regex(r"^/v4/locations:.*\.json$"))
        .and(query_param("usage", "short"))
        .respond_with(ResponseTemplate::new(200).set_body_string(mock_locations_response.clone()))
        .expect(1)
        .named("locations")
        .mount(&mock_winnipeg_transit_api)
        .await;

    let mock_stops_response = fs::read_to_string("tests/fixtures/stops/stops.json")
        .expect("Failed to read stops fixture");

    Mock::given(method("GET"))
        .and(path_regex(r"^/v4/stops.json$"))
        .and(query_param("lat", "49.88895"))
        .and(query_param("lon", "-97.13424"))
        .and(query_param("distance", "500"))
        .and(query_param("usage", "short"))
        .respond_with(ResponseTemplate::new(200).set_body_string(mock_stops_response.clone()))
        .expect(1)
        .named("stops")
        .mount(&mock_winnipeg_transit_api)
        .await;

    let stops: serde_json::Value =
        serde_json::from_str(&mock_stops_response).expect("Failed to parse stops fixture as JSON");

    let mut mock_routes: Vec<(String, String, String)> = Vec::new();

    for stop in stops["stops"].as_array().unwrap().iter().take(10) {
        let stop_key = stop["key"].as_u64().unwrap().to_string();
        let mock_routes_response = fs::read_to_string(format!(
            "tests/fixtures/stops/routes/stop_{}.json",
            stop_key
        ))
        .unwrap_or_else(|_| panic!("Failed to read routes fixture for stop {}", stop_key));

        let mock_route_path = format!("/v4/routes.json?stop={}", stop_key);
        mock_routes.push((
            mock_route_path.clone(),
            mock_routes_response.clone(),
            stop_key.clone(),
        ));

        Mock::given(method("GET"))
            .and(path("/v4/routes.json"))
            .and(query_param("stop", stop_key.as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_string(mock_routes_response))
            .expect(1)
            .named(format!("routes for stop {}", stop_key))
            .mount(&mock_winnipeg_transit_api)
            .await;
    }

    let response = get(
        "/twilio?Body=Stops Union Station&From=approved&To=textabus&MessageSid=SM1849",
        InjectableServices {
            db: db.clone(),
            twilio_address: None,
            winnipeg_transit_api_address: Some(mock_winnipeg_transit_api.uri()),
        },
    )
    .await;

    assert!(response.is_ok(), "Failed to execute request");

    let response = response.unwrap();

    assert!(response.status().is_success());
    assert_eq!(response.headers()["content-type"], "text/xml");

    let document = Document::from(response.text().await.unwrap().as_str());
    let body = &document.find(Name("body")).next().unwrap().text();

    let expected_body = indoc! {"
        Stops near Via Rail Station (Union Station) (123 MainSt)

        10625 NB Main@Broadway (Union Station) BLUE 14 19 47 53 54 55 57 59 68

        10641 SB Main@Broadway (Union Station) BLUE 14 19 23 47 53 54 55 57 59 65 66 68

        11052 WB Broadway@Main 34 65 66

        11010 NB Fort@Broadway 34

        10901 SB Israel Asper@Canadian Museum for Human Rights 38

        10902 NB Israel Asper@Canadian Museum for Human Rights 38

        10624 NB Main@Assiniboine BLUE 14 19 47 53 54 55 57 59 68

        10830 NB Fort@Assiniboine 23

        10907 EB Forks Market@The Forks Market 38

        10639 SB Main@St. Mary BLUE 14 19 34 47 53 54 55 57 59 68
    "};

    assert_that(body).contains(expected_body);

    let [incoming_message, outgoing_message]: [Message; 2] =
        sqlx::query_as("SELECT * FROM messages ORDER BY created_at")
            .fetch_all(&db)
            .await
            .expect("Failed to fetch messages")
            .try_into()
            .expect("Expected exactly 2 messages");

    assert_eq!(incoming_message.body, "Stops Union Station");

    assert_eq!(outgoing_message.body, expected_body);

    let api_responses: Vec<ApiResponse> = sqlx::query_as("SELECT * FROM api_responses")
        .fetch_all(&db)
        .await
        .expect("Failed to fetch API responses");

    let locations_response = api_responses
        .first()
        .expect("Expected persisted locations response");

    assert_eq!(locations_response.message_id, incoming_message.id);
    assert_eq!(locations_response.body, mock_locations_response);
    assert_starts_with!(
        locations_response.query,
        format!("/v4/locations:Union Station.json?usage=short")
    );

    let stops_response = api_responses
        .get(1)
        .expect("Expected persisted stops response");

    assert_eq!(stops_response.message_id, incoming_message.id);
    assert_eq!(stops_response.body, mock_stops_response);
    assert_starts_with!(
        stops_response.query,
        format!("/v4/stops.json?lat=49.88895&lon=-97.13424&distance=500&usage=short")
    );

    let routes_responses: Vec<&ApiResponse> = api_responses.iter().skip(2).collect();

    for (index, (path, data, stop)) in mock_routes.iter().enumerate() {
        let route_response = routes_responses
            .get(index)
            .unwrap_or_else(|| panic!("Expected persisted route response for stop {}", stop));

        assert_eq!(route_response.message_id, incoming_message.id);
        assert_eq!(route_response.body, *data);
        assert_starts_with!(route_response.query, *path);
    }
}

#[sqlx::test(fixtures("numbers-approved"))]
async fn stops_handles_an_empty_locations_response(db: PgPool) {
    let mock_winnipeg_transit_api: MockServer = MockServer::start().await;

    let mock_locations_response = fs::read_to_string("tests/fixtures/stops/locations-none.json")
        .expect("Failed to read locations fixture");

    Mock::given(method("GET"))
        .and(path_regex(r"^/v4/locations:.*\.json$"))
        .and(query_param("usage", "short"))
        .respond_with(ResponseTemplate::new(200).set_body_string(mock_locations_response.clone()))
        .expect(1)
        .named("locations")
        .mount(&mock_winnipeg_transit_api)
        .await;

    let response = get(
        "/twilio?Body=stops acab&From=approved&To=textabus&MessageSid=SM1849",
        InjectableServices {
            db: db.clone(),
            twilio_address: None,
            winnipeg_transit_api_address: Some(mock_winnipeg_transit_api.uri()),
        },
    )
    .await;

    assert!(response.is_ok(), "Failed to execute request");

    let response = response.unwrap();

    assert!(response.status().is_success());
    assert_eq!(response.headers()["content-type"], "text/xml");

    let document = Document::from(response.text().await.unwrap().as_str());
    let body = &document.find(Name("body")).next().unwrap().text();

    let expected_body = indoc! {"
        No locations found for acab
    "};

    assert_that(body).contains(expected_body);

    let api_response: ApiResponse = sqlx::query_as("SELECT * FROM api_responses LIMIT 1")
        .fetch_one(&db)
        .await
        .expect("Failed to fetch API response");

    assert_eq!(api_response.body, mock_locations_response);
    assert_starts_with!(api_response.query, "/v4/locations:acab.json?usage=short");
}

#[sqlx::test(fixtures("numbers-approved"))]
async fn stops_handles_an_empty_stops_response(db: PgPool) {
    let mock_winnipeg_transit_api: MockServer = MockServer::start().await;

    let mock_locations_response =
        fs::read_to_string("tests/fixtures/stops/locations-no-stops.json")
            .expect("Failed to read locations fixture");

    Mock::given(method("GET"))
        .and(path_regex(r"^/v4/locations:.*\.json$"))
        .and(query_param("usage", "short"))
        .respond_with(ResponseTemplate::new(200).set_body_string(mock_locations_response.clone()))
        .expect(1)
        .named("locations")
        .mount(&mock_winnipeg_transit_api)
        .await;

    let mock_stops_response = fs::read_to_string("tests/fixtures/stops/stops-none.json")
        .expect("Failed to read stops fixture");

    Mock::given(method("GET"))
        .and(path_regex(r"^/v4/stops.json$"))
        .respond_with(ResponseTemplate::new(200).set_body_string(mock_stops_response.clone()))
        .expect(1)
        .named("stops")
        .mount(&mock_winnipeg_transit_api)
        .await;

    let response = get(
        "/twilio?Body=stops assiniboia downs&From=approved&To=textabus&MessageSid=SM1849",
        InjectableServices {
            db: db.clone(),
            twilio_address: None,
            winnipeg_transit_api_address: Some(mock_winnipeg_transit_api.uri()),
        },
    )
    .await;

    assert!(response.is_ok(), "Failed to execute request");

    let response = response.unwrap();

    assert!(response.status().is_success());
    assert_eq!(response.headers()["content-type"], "text/xml");

    let document = Document::from(response.text().await.unwrap().as_str());
    let body = &document.find(Name("body")).next().unwrap().text();

    let expected_body = indoc! {"
        No stops found within 500m of Assiniboine Downs (3975 PortageAve)
    "};

    assert_that(body).contains(expected_body);

    let api_responses_record_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_responses")
        .fetch_one(&db)
        .await
        .expect("Failed to fetch api_responses count");

    assert_eq!(api_responses_record_count, 2);
}
