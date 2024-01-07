use indoc::indoc;
use select::{document::Document, predicate::Name};
use serde_json::json;
use speculoos::prelude::*;
use textabus::{app, InjectableServices};
use tokio::net::TcpListener;
use wiremock::matchers::{any, method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn root_serves_placeholder() {
    let response = get(
        "/",
        InjectableServices {
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

#[tokio::test]
async fn twilio_serves_placeholder_with_unknown_body() {
    let response = get(
        "/twilio?Body=wha",
        InjectableServices {
            winnipeg_transit_api_address: None,
        },
    )
    .await
    .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(response.headers()["content-type"], "text/xml");

    let document = Document::from(response.text().await.unwrap().as_str());

    assert_that(&document.find(Name("body")).next().unwrap().text()).contains("textabus");
}

#[tokio::test]
async fn stop_number_returns_stop_name() {
    let mock_winnipeg_transit_api = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path_regex(r"^/v3/stops/.*/schedule.json$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(
            {
                "stop-schedule": {
                    "stop": {
                        "key": 10064,
                        "name": "NB Osborne@Glasgow",
                        "number": 10064,
                        "direction": "Northbound",
                        "side": "Nearside",
                        "street": {
                            "key": 2715,
                            "name": "OsborneSt",
                            "type": "Street"
                        },
                        "cross-street": {
                            "key": 1486,
                            "name": "GlasgowAve",
                            "type": "Avenue"
                        },
                        "centre": {
                            "utm": {
                                "zone": "14U",
                                "x": 633838,
                                "y": 5525742
                            },
                            "geographic": {
                                "latitude": "49.86912",
                                "longitude": "-97.1375"
                            }
                        }
                    },
                    "route-schedules": [
                        {
                            "route": {
                                "key": 16,
                                "number": 16,
                                "name": "Selkirk-Osborne",
                                "customer-type": "regular",
                                "coverage": "regular",
                                "badge-label": 16,
                                "badge-style": {
                                    "class-names": {
                                        "class-name": [
                                            "badge-label",
                                            "regular"
                                        ]
                                    },
                                    "background-color": "#ffffff",
                                    "border-color": "#d9d9d9",
                                    "color": "#000000"
                                }
                            },
                            "scheduled-stops": [
                                {
                                    "key": "25591949-52",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-06T16:16:31",
                                            "estimated": "2024-01-06T16:29:39"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-06T16:16:31",
                                            "estimated": "2024-01-06T16:29:39"
                                        }
                                    },
                                    "variant": {
                                        "key": "16-0-M",
                                        "name": "Via Manitoba"
                                    },
                                    "bus": {
                                        "key": 820,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25591950-42",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-06T16:36:31",
                                            "estimated": "2024-01-06T16:36:31"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-06T16:36:31",
                                            "estimated": "2024-01-06T16:36:31"
                                        }
                                    },
                                    "variant": {
                                        "key": "16-0-B",
                                        "name": "Via Burrows"
                                    },
                                    "bus": {
                                        "key": 730,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25591951-52",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-06T16:57:31",
                                            "estimated": "2024-01-06T16:57:31"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-06T16:57:31",
                                            "estimated": "2024-01-06T16:57:31"
                                        }
                                    },
                                    "variant": {
                                        "key": "16-0-M",
                                        "name": "Via Manitoba"
                                    },
                                    "bus": {
                                        "key": 817,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25591952-42",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-06T17:19:31",
                                            "estimated": "2024-01-06T17:19:31"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-06T17:19:31",
                                            "estimated": "2024-01-06T17:19:31"
                                        }
                                    },
                                    "variant": {
                                        "key": "16-0-B",
                                        "name": "Via Burrows"
                                    },
                                    "bus": {
                                        "key": 442,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25591953-52",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-06T17:43:31",
                                            "estimated": "2024-01-06T17:43:31"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-06T17:43:31",
                                            "estimated": "2024-01-06T17:43:31"
                                        }
                                    },
                                    "variant": {
                                        "key": "16-0-M",
                                        "name": "Via Manitoba"
                                    },
                                    "bus": {
                                        "key": 811,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                }
                            ]
                        }
                    ]
                },
                "query-time": "2024-01-06T16:06:26"
            }
        )))
        .expect(1)
        .mount(&mock_winnipeg_transit_api)
        .await;

    let response = get(
        "/twilio?Body=10064",
        InjectableServices {
            winnipeg_transit_api_address: Some(mock_winnipeg_transit_api.uri()),
        },
    )
    .await
    .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(response.headers()["content-type"], "text/xml");

    let document = Document::from(response.text().await.unwrap().as_str());
    let body = &document.find(Name("body")).next().unwrap().text();

    assert_that(body).contains(indoc! {"
        10064 NB Osborne@Glasgow
        4:29p 16 Via Manitoba
        4:36p 16 Via Burrows
        4:57p 16 Via Manitoba
        5:19p 16 Via Burrows
        5:43p 16 Via Manitoba"});
}

async fn get(
    path: &str,
    mut services: InjectableServices,
) -> Result<reqwest::Response, reqwest::Error> {
    if services.winnipeg_transit_api_address.is_none() {
        let mock_winnipeg_transit_api = MockServer::start().await;

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(0)
            .named("Mock Winnipeg Transit API")
            .mount(&mock_winnipeg_transit_api)
            .await;

        services = InjectableServices {
            winnipeg_transit_api_address: Some("http://localhost:1313".to_string()),
        };
    }

    let app_address = spawn_app(services).await.address;

    let client = reqwest::Client::new();
    let url = format!("{}{}", app_address, path);

    client.get(&url).send().await
}

struct TestApp {
    pub address: String,
}

async fn spawn_app(services: InjectableServices) -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    tokio::spawn(async move {
        axum::serve(listener, app(services).await.into_make_service())
            .await
            .unwrap();
    });

    TestApp { address }
}
