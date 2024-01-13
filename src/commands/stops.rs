use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

use crate::config::Config;

const STOPS_DISTANCE: usize = 500;

pub async fn handle_stops_request(
    config: &Config,
    winnipeg_transit_api_address: String,
    location: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();

    let api_key = config.winnipeg_transit_api_key.clone();

    let locations_url = format!(
        "{}/v3/locations:{}.json?api-key={}",
        winnipeg_transit_api_address, location, api_key
    );

    log::trace!("locations URL: {}", locations_url);

    let locations_response = client.get(&locations_url).send().await?;
    let locations_response_text = locations_response.text().await?;

    let locations_response: LocationResponse = match serde_json::from_str(&locations_response_text)
    {
        Ok(response) => response,
        Err(err) => {
            log::error!("Error parsing locations response: {}", err);
            log::error!("Response: {}", locations_response_text);
            return Err(Box::new(err));
        }
    };

    let latitude = &locations_response.locations[0]
        .address
        .centre
        .geographic
        .latitude;

    let longitude = &locations_response.locations[0]
        .address
        .centre
        .geographic
        .longitude;

    let location_address = format!(
        "{} {}",
        locations_response.locations[0].address.street_number,
        locations_response.locations[0].address.street.name
    );

    let stops_url = format!(
        "{}/v3/stops.json?lat={}&lon={}&distance={}&api-key={}",
        winnipeg_transit_api_address, latitude, longitude, STOPS_DISTANCE, api_key
    );

    log::trace!("stops URL: {}", stops_url);

    let stops_response_text = client.get(&stops_url).send().await?.text().await?;

    let stops_response: StopsResponse = match serde_json::from_str(&stops_response_text) {
        Ok(response) => response,
        Err(err) => {
            log::error!("Error parsing stops response: {}", err);
            log::error!("Response: {}", stops_response_text);
            return Err(Box::new(err));
        }
    };

    let mut response = format!(
        "Stops near {} ({})\n",
        &locations_response.locations[0].name, location_address
    );

    for stop in &stops_response.stops {
        let routes_url = format!(
            "{}/v3/routes.json?stop={}&api-key={}",
            winnipeg_transit_api_address, stop.number, api_key
        );

        log::trace!("routes URL: {}", routes_url);

        let routes_response_text = client.get(&routes_url).send().await?.text().await?;

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

        let routes: Vec<String> = routes_response
            .routes
            .iter()
            .map(|route| match &route.number {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => panic!("Unexpected type parsing route number"),
            })
            .collect();

        response += &format!("{} {} {}\n", stop.number, stop.name, routes.join(" "));
    }

    Ok(response)
}

#[derive(Deserialize)]
struct LocationResponse {
    locations: Vec<Location>,
}

#[derive(Deserialize)]
struct Location {
    name: String,
    address: Address,
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
