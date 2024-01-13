use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use sqlx::{types::Uuid, PgPool};

use crate::config::Config;

const STOPS_DISTANCE: usize = 500;
const MAXIMUM_STOPS_TO_RETURN: usize = 10;

pub async fn handle_stops_request(
    config: &Config,
    winnipeg_transit_api_address: String,
    location: &str,
    maybe_incoming_message_id: Option<Uuid>,
    db: &PgPool,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();

    let api_key = config.winnipeg_transit_api_key.clone();

    let locations_query = format!("/v3/locations:{}.json?usage=short", location);

    let locations_url = format!(
        "{}{}&api-key={}",
        winnipeg_transit_api_address, locations_query, api_key
    );

    log::trace!("locations URL: {}", locations_url);

    let locations_response = client.get(&locations_url).send().await?;

    let locations_response_text = locations_response.text().await?;

    let locations_api_response_insertion_result = sqlx::query(
        r#"
        INSERT INTO api_responses (id, body, query, message_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(locations_response_text.clone())
    .bind(locations_query)
    .bind(maybe_incoming_message_id)
    .bind(Utc::now().naive_utc())
    .bind(Utc::now().naive_utc())
    .execute(db)
    .await;

    if let Err(e) = locations_api_response_insertion_result {
        log::error!("Failed to insert locations API response: {}", e);
    }

    let (location_name, latitude, longitude) = extract_location_details(&locations_response_text)?;

    let stops_query = format!(
        "/v3/stops.json?lat={}&lon={}&distance={}&usage=short",
        latitude, longitude, STOPS_DISTANCE
    );

    let stops_url = format!(
        "{}{}&api-key={}",
        winnipeg_transit_api_address, stops_query, api_key
    );

    log::trace!("stops URL: {}", stops_url);

    let stops_response_text = client.get(&stops_url).send().await?.text().await?;

    let stops_api_response_insertion_result = sqlx::query(
        r#"
        INSERT INTO api_responses (id, body, query, message_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(stops_response_text.clone())
    .bind(stops_query)
    .bind(maybe_incoming_message_id)
    .bind(Utc::now().naive_utc())
    .bind(Utc::now().naive_utc())
    .execute(db)
    .await;

    if let Err(e) = stops_api_response_insertion_result {
        log::error!("Failed to insert locations API response: {}", e);
    }

    let stops_response: StopsResponse = match serde_json::from_str(&stops_response_text) {
        Ok(response) => response,
        Err(err) => {
            log::error!("Error parsing stops response: {}", err);
            log::error!("Response: {}", stops_response_text);
            return Err(Box::new(err));
        }
    };

    let mut response = format!("Stops near {}\n", location_name);

    for stop in stops_response.stops.iter().take(MAXIMUM_STOPS_TO_RETURN) {
        let routes_query = format!("/v3/routes.json?stop={}", stop.number);

        let routes_url = format!(
            "{}{}&api-key={}",
            winnipeg_transit_api_address, routes_query, api_key
        );

        log::trace!("routes URL: {}", routes_url);

        let routes_response_text = client.get(&routes_url).send().await?.text().await?;

        let route_stops_api_response_insertion_result = sqlx::query(
            r#"
            INSERT INTO api_responses (id, body, query, message_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(routes_response_text.clone())
        .bind(routes_query)
        .bind(maybe_incoming_message_id)
        .bind(Utc::now().naive_utc())
        .bind(Utc::now().naive_utc())
        .execute(db)
        .await;

        if let Err(e) = route_stops_api_response_insertion_result {
            log::error!(
                "Failed to insert routes API response for stop {}: {}",
                stop.number,
                e
            );
        }

        let routes_response: RoutesResponse = match serde_json::from_str(&routes_response_text) {
            Ok(response) => response,
            Err(err) => {
                log::error!(
                    "Error parsing routes response for stop {}: {}",
                    stop.number,
                    err
                );
                log::error!("Response: {}", routes_response_text);
                return Err(Box::new(err));
            }
        };

        if routes_response.routes.is_empty() {
            continue;
        }

        let mut routes: Vec<String> = routes_response
            .routes
            .iter()
            .map(|route| match &route.number {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => panic!("Unexpected type parsing route number"),
            })
            .collect();

        routes.sort_by(|a, b| {
            let a_is_numeric = a.chars().all(char::is_numeric);
            let b_is_numeric = b.chars().all(char::is_numeric);

            if a_is_numeric && b_is_numeric {
                a.parse::<u64>().unwrap().cmp(&b.parse::<u64>().unwrap())
            } else if a_is_numeric {
                std::cmp::Ordering::Greater
            } else if b_is_numeric {
                std::cmp::Ordering::Less
            } else {
                a.cmp(b)
            }
        });

        response += &format!("\n{} {} {}\n", stop.number, stop.name, routes.join(" "));
    }

    Ok(response)
}

fn extract_location_details(
    locations_response_text: &str,
) -> Result<(String, String, String), Box<dyn std::error::Error>> {
    let location_name;
    let latitude;
    let longitude;

    let locations_response: LocationResponse = match serde_json::from_str(&locations_response_text)
    {
        Ok(response) => response,
        Err(err) => {
            log::error!("Error parsing locations response: {}", err);
            log::error!("Response: {}", locations_response_text);
            return Err(Box::new(err));
        }
    };

    match &locations_response.locations[0] {
        Location::Address(address) => {
            location_name = format!("{} {}", address.street_number, address.street.name);
            latitude = address.centre.geographic.latitude.clone();
            longitude = address.centre.geographic.longitude.clone();
        }
        Location::Intersection(intersection) => {
            location_name = format!(
                "{}@{}",
                intersection.street.name, intersection.cross_street.name
            );
            latitude = intersection.centre.geographic.latitude.clone();
            longitude = intersection.centre.geographic.longitude.clone();
        }
        Location::Monument(monument) => {
            let monument_address = format!(
                "{} {}",
                monument.address.street_number, monument.address.street.name
            );
            location_name = format!("{} ({})", monument.name.clone(), monument_address);
            latitude = monument.address.centre.geographic.latitude.clone();
            longitude = monument.address.centre.geographic.longitude.clone();
        }
    }

    Ok((location_name, latitude, longitude))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_monument_details() {
        let locations_response_text = include_str!("../../tests/fixtures/stops/locations.json");

        let result = extract_location_details(locations_response_text);
        assert!(result.is_ok());

        let (location_name, latitude, longitude) = result.unwrap();
        assert_eq!(
            location_name,
            "Via Rail Station (Union Station) (123 Main Street)"
        );
        assert_eq!(latitude, "49.88895");
        assert_eq!(longitude, "-97.13424");
    }

    #[test]
    fn test_extract_address_details() {
        let locations_response_text =
            include_str!("../../tests/fixtures/stops/locations-address.json");

        let result = extract_location_details(locations_response_text);
        assert!(result.is_ok());

        let (location_name, latitude, longitude) = result.unwrap();
        assert_eq!(location_name, "245 SmithSt");
        assert_eq!(latitude, "49.89218");
        assert_eq!(longitude, "-97.14084");
    }

    #[test]
    fn test_extract_intersection_details() {
        let locations_response_text =
            include_str!("../../tests/fixtures/stops/locations-intersection.json");

        let result = extract_location_details(locations_response_text);
        assert!(result.is_ok());

        let (location_name, latitude, longitude) = result.unwrap();
        assert_eq!(location_name, "PortageAve@MainSt");
        assert_eq!(latitude, "49.89553");
        assert_eq!(longitude, "-97.13848");
    }
}

#[derive(Deserialize)]
struct LocationResponse {
    locations: Vec<Location>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum Location {
    Address(Address),
    Intersection(Intersection),
    Monument(Monument),
}

#[derive(Deserialize)]
struct Centre {
    geographic: Geographic,
}

#[derive(Deserialize)]
struct Geographic {
    latitude: String,
    longitude: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Address {
    centre: Centre,
    street_number: u64,
    street: Street,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Intersection {
    centre: Centre,
    street: Street,
    cross_street: Street,
}

#[derive(Deserialize)]
struct Monument {
    name: String,
    address: Address,
}

#[derive(Deserialize)]
struct Street {
    name: String,
}

#[derive(Deserialize)]
struct StopsResponse {
    stops: Vec<Stop>,
}

#[derive(Deserialize)]
struct Stop {
    number: u64,
    name: String,
}

#[derive(Deserialize)]
struct RoutesResponse {
    routes: Vec<Route>,
}

#[derive(Deserialize)]
struct Route {
    number: Value,
}
