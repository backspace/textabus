use serde::Deserialize;
use serde_json::{Number, Value};
use sqlx::{types::Uuid, PgPool};

use crate::{commands::StopsCommand, config::Config, odws::fetch_from_odws};

const STOPS_DISTANCE: usize = 500;
const MAXIMUM_STOPS_TO_RETURN: usize = 10;

pub async fn handle_stops_request(
    command: StopsCommand,
    config: &Config,
    winnipeg_transit_api_address: String,
    maybe_incoming_message_id: Option<Uuid>,
    db: &PgPool,
) -> Result<String, Box<dyn std::error::Error>> {
    let locations_query = format!("/v4/locations:{}.json?usage=short", command.location);
    log::trace!("locations URL: {}", locations_query);

    let (_locations_response_status, locations_response_text) = fetch_from_odws(
        locations_query,
        config,
        winnipeg_transit_api_address.clone(),
        maybe_incoming_message_id,
        db,
    )
    .await;

    let (location_name, latitude, longitude) =
        match extract_location_details(&locations_response_text) {
            Ok(details) => details,
            Err(_) => return Ok(format!("No locations found for {}", command.location).to_string()),
        };

    let stops_query = format!(
        "/v4/stops.json?lat={}&lon={}&distance={}&usage=short",
        latitude, longitude, STOPS_DISTANCE
    );

    log::trace!("stops URL: {}", stops_query);

    let (_stops_response_status, stops_response_text) = fetch_from_odws(
        stops_query,
        config,
        winnipeg_transit_api_address.clone(),
        maybe_incoming_message_id,
        db,
    )
    .await;

    let stops_response: StopsResponse = match serde_json::from_str(&stops_response_text) {
        Ok(response) => response,
        Err(err) => {
            log::error!("Error parsing stops response: {}", err);
            log::error!("Response: {}", stops_response_text);
            return Err(Box::new(err));
        }
    };

    if stops_response.stops.is_empty() {
        return Ok(format!(
            "No stops found within {}m of {}",
            STOPS_DISTANCE, location_name
        )
        .to_string());
    }

    let mut response = format!("Stops near {}\n", location_name);

    for stop in stops_response.stops.iter().take(MAXIMUM_STOPS_TO_RETURN) {
        let routes_query = format!("/v4/routes.json?stop={}", stop.number);

        log::trace!("routes URL: {}", routes_query);

        let (_routes_response_status, routes_response_text) = fetch_from_odws(
            routes_query,
            config,
            winnipeg_transit_api_address.clone(),
            maybe_incoming_message_id,
            db,
        )
        .await;

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
) -> Result<(String, Number, Number), Box<dyn std::error::Error>> {
    let location_name;
    let latitude;
    let longitude;

    let locations_response: LocationResponse = match serde_json::from_str(locations_response_text) {
        Ok(response) => response,
        Err(err) => {
            log::error!("Error parsing locations response: {}", err);
            log::error!("Response: {}", locations_response_text);
            return Err(Box::new(err));
        }
    };

    if locations_response.locations.is_empty() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No locations found",
        )));
    }

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
        assert_eq!(latitude.to_string(), "49.88895");
        assert_eq!(longitude.to_string(), "-97.13424");
    }

    #[test]
    fn test_extract_address_details() {
        let locations_response_text =
            include_str!("../../tests/fixtures/stops/locations-address.json");

        let result = extract_location_details(locations_response_text);
        assert!(result.is_ok());

        let (location_name, latitude, longitude) = result.unwrap();
        assert_eq!(location_name, "245 SmithSt");
        assert_eq!(latitude.to_string(), "49.89218");
        assert_eq!(longitude.to_string(), "-97.14084");
    }

    #[test]
    fn test_extract_intersection_details() {
        let locations_response_text =
            include_str!("../../tests/fixtures/stops/locations-intersection.json");

        let result = extract_location_details(locations_response_text);
        assert!(result.is_ok());

        let (location_name, latitude, longitude) = result.unwrap();
        assert_eq!(location_name, "PortageAve@MainSt");
        assert_eq!(latitude.to_string(), "49.89553");
        assert_eq!(longitude.to_string(), "-97.13848");
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
    latitude: Number,
    longitude: Number,
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
