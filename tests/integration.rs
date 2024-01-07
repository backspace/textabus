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
                        "key": 10619,
                        "name": "WB Graham@Vaughan (The Bay)",
                        "number": 10619,
                        "direction": "Westbound",
                        "side": "Nearside",
                        "street": {
                            "key": 1533,
                            "name": "GrahamAve",
                            "type": "Avenue"
                        },
                        "cross-street": {
                            "key": 3716,
                            "name": "VaughanSt",
                            "type": "Street"
                        },
                        "centre": {
                            "utm": {
                                "zone": "14U",
                                "x": 632952,
                                "y": 5528122
                            },
                            "geographic": {
                                "latitude": "49.89071",
                                "longitude": "-97.149"
                            }
                        }
                    },
                    "route-schedules": [
                        {
                            "route": {
                                "key": 33,
                                "number": 33,
                                "name": "Maples",
                                "customer-type": "regular",
                                "coverage": "regular",
                                "badge-label": 33,
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
                                    "key": "25594106-6",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T12:37:57",
                                            "estimated": "2024-01-07T12:37:57"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T12:37:57",
                                            "estimated": "2024-01-07T12:37:57"
                                        }
                                    },
                                    "variant": {
                                        "key": "33-0-M",
                                        "name": "Via Mapleglen"
                                    },
                                    "bus": {
                                        "key": 602,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594107-6",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T13:08:57",
                                            "estimated": "2024-01-07T13:08:57"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T13:08:57",
                                            "estimated": "2024-01-07T13:08:57"
                                        }
                                    },
                                    "variant": {
                                        "key": "33-0-J",
                                        "name": "Via Jefferson"
                                    },
                                    "bus": {
                                        "key": 870,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594108-6",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T13:39:57",
                                            "estimated": "2024-01-07T13:39:57"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T13:39:57",
                                            "estimated": "2024-01-07T13:39:57"
                                        }
                                    },
                                    "variant": {
                                        "key": "33-0-M",
                                        "name": "Via Mapleglen"
                                    },
                                    "bus": {
                                        "key": 402,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594091-6",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T14:10:57",
                                            "estimated": "2024-01-07T14:10:57"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T14:10:57",
                                            "estimated": "2024-01-07T14:10:57"
                                        }
                                    },
                                    "variant": {
                                        "key": "33-0-J",
                                        "name": "Via Jefferson"
                                    },
                                    "bus": {
                                        "key": 602,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                }
                            ]
                        },
                        {
                            "route": {
                                "key": 45,
                                "number": 45,
                                "name": "Talbot",
                                "customer-type": "regular",
                                "coverage": "regular",
                                "badge-label": 45,
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
                                    "key": "25595206-47",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T12:54:01",
                                            "estimated": "2024-01-07T12:59:01"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T12:54:01",
                                            "estimated": "2024-01-07T12:59:01"
                                        }
                                    },
                                    "variant": {
                                        "key": "45-1-D",
                                        "name": "Downtown"
                                    },
                                    "bus": {
                                        "key": 869,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25595207-47",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T13:34:01",
                                            "estimated": "2024-01-07T13:34:01"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T13:34:01",
                                            "estimated": "2024-01-07T13:34:01"
                                        }
                                    },
                                    "variant": {
                                        "key": "45-1-D",
                                        "name": "Downtown"
                                    },
                                    "bus": {
                                        "key": 854,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25595208-47",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T14:14:01",
                                            "estimated": "2024-01-07T14:14:01"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T14:14:01",
                                            "estimated": "2024-01-07T14:14:01"
                                        }
                                    },
                                    "variant": {
                                        "key": "45-1-D",
                                        "name": "Downtown"
                                    },
                                    "bus": {
                                        "key": 869,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                }
                            ]
                        },
                        {
                            "route": {
                                "key": "BLUE",
                                "number": "BLUE",
                                "customer-type": "regular",
                                "coverage": "rapid transit",
                                "badge-label": "B",
                                "badge-style": {
                                    "class-names": {
                                        "class-name": [
                                            "badge-label",
                                            "rapid-transit"
                                        ]
                                    },
                                    "background-color": "#0060a9",
                                    "border-color": "#0060a9",
                                    "color": "#ffffff"
                                }
                            },
                            "scheduled-stops": [
                                {
                                    "key": "25594882-35",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T12:10:35",
                                            "estimated": "2024-01-07T12:19:13"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T12:10:35",
                                            "estimated": "2024-01-07T12:19:13"
                                        }
                                    },
                                    "variant": {
                                        "key": "BLUE-1-D",
                                        "name": "Downtown"
                                    },
                                    "bus": {
                                        "key": 382,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594883-22",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T12:22:35",
                                            "estimated": "2024-01-07T12:22:35"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T12:22:35",
                                            "estimated": "2024-01-07T12:22:35"
                                        }
                                    },
                                    "variant": {
                                        "key": "BLUE-1-D",
                                        "name": "Downtown"
                                    },
                                    "bus": {
                                        "key": 380,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594884-35",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T12:33:35",
                                            "estimated": "2024-01-07T12:33:35"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T12:33:35",
                                            "estimated": "2024-01-07T12:33:35"
                                        }
                                    },
                                    "variant": {
                                        "key": "BLUE-1-D",
                                        "name": "Downtown"
                                    },
                                    "bus": {
                                        "key": 395,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594885-22",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T12:45:35",
                                            "estimated": "2024-01-07T12:45:35"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T12:45:35",
                                            "estimated": "2024-01-07T12:45:35"
                                        }
                                    },
                                    "variant": {
                                        "key": "BLUE-1-D",
                                        "name": "Downtown"
                                    },
                                    "bus": {
                                        "key": 374,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594886-35",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T12:56:35",
                                            "estimated": "2024-01-07T12:56:35"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T12:56:35",
                                            "estimated": "2024-01-07T12:56:35"
                                        }
                                    },
                                    "variant": {
                                        "key": "BLUE-1-D",
                                        "name": "Downtown"
                                    },
                                    "bus": {
                                        "key": 388,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594887-22",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T13:08:35",
                                            "estimated": "2024-01-07T13:08:35"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T13:08:35",
                                            "estimated": "2024-01-07T13:08:35"
                                        }
                                    },
                                    "variant": {
                                        "key": "BLUE-1-D",
                                        "name": "Downtown"
                                    },
                                    "bus": {
                                        "key": 381,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594888-35",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T13:19:35",
                                            "estimated": "2024-01-07T13:19:35"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T13:19:35",
                                            "estimated": "2024-01-07T13:19:35"
                                        }
                                    },
                                    "variant": {
                                        "key": "BLUE-1-D",
                                        "name": "Downtown"
                                    },
                                    "bus": {
                                        "key": 387,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594889-22",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T13:31:35",
                                            "estimated": "2024-01-07T13:31:35"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T13:31:35",
                                            "estimated": "2024-01-07T13:31:35"
                                        }
                                    },
                                    "variant": {
                                        "key": "BLUE-1-D",
                                        "name": "Downtown"
                                    },
                                    "bus": {
                                        "key": 378,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594890-35",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T13:42:35",
                                            "estimated": "2024-01-07T13:42:35"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T13:42:35",
                                            "estimated": "2024-01-07T13:42:35"
                                        }
                                    },
                                    "variant": {
                                        "key": "BLUE-1-D",
                                        "name": "Downtown"
                                    },
                                    "bus": {
                                        "key": 382,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594891-22",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T13:54:35",
                                            "estimated": "2024-01-07T13:54:35"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T13:54:35",
                                            "estimated": "2024-01-07T13:54:35"
                                        }
                                    },
                                    "variant": {
                                        "key": "BLUE-1-D",
                                        "name": "Downtown"
                                    },
                                    "bus": {
                                        "key": 380,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594892-35",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T14:05:35",
                                            "estimated": "2024-01-07T14:05:35"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T14:05:35",
                                            "estimated": "2024-01-07T14:05:35"
                                        }
                                    },
                                    "variant": {
                                        "key": "BLUE-1-D",
                                        "name": "Downtown"
                                    },
                                    "bus": {
                                        "key": 395,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                }
                            ]
                        },
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
                                    "key": "25594562-50",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T12:18:00",
                                            "estimated": "2024-01-07T12:18:08"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T12:18:00",
                                            "estimated": "2024-01-07T12:18:08"
                                        }
                                    },
                                    "variant": {
                                        "key": "16-1-V",
                                        "name": "St Vital Ctr"
                                    },
                                    "bus": {
                                        "key": 344,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594563-51",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T12:39:00",
                                            "estimated": "2024-01-07T12:39:00"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T12:39:00",
                                            "estimated": "2024-01-07T12:39:00"
                                        }
                                    },
                                    "variant": {
                                        "key": "16-1-##",
                                        "name": "Southdale Ctr"
                                    },
                                    "bus": {
                                        "key": 888,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594564-50",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T13:00:00",
                                            "estimated": "2024-01-07T13:00:00"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T13:00:00",
                                            "estimated": "2024-01-07T13:00:00"
                                        }
                                    },
                                    "variant": {
                                        "key": "16-1-V",
                                        "name": "St Vital Ctr"
                                    },
                                    "bus": {
                                        "key": 450,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594565-51",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T13:21:00",
                                            "estimated": "2024-01-07T13:21:00"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T13:21:00",
                                            "estimated": "2024-01-07T13:21:00"
                                        }
                                    },
                                    "variant": {
                                        "key": "16-1-##",
                                        "name": "Southdale Ctr"
                                    },
                                    "bus": {
                                        "key": 172,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594566-50",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T13:42:00",
                                            "estimated": "2024-01-07T13:42:00"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T13:42:00",
                                            "estimated": "2024-01-07T13:42:00"
                                        }
                                    },
                                    "variant": {
                                        "key": "16-1-V",
                                        "name": "St Vital Ctr"
                                    },
                                    "bus": {
                                        "key": 712,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594569-51",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T14:03:00",
                                            "estimated": "2024-01-07T14:03:00"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T14:03:00",
                                            "estimated": "2024-01-07T14:03:00"
                                        }
                                    },
                                    "variant": {
                                        "key": "16-1-##",
                                        "name": "Southdale Ctr"
                                    },
                                    "bus": {
                                        "key": 606,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                }
                            ]
                        },
                        {
                            "route": {
                                "key": 17,
                                "number": 17,
                                "name": "McGregor",
                                "customer-type": "regular",
                                "coverage": "regular",
                                "badge-label": 17,
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
                                    "key": "25594688-63",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T12:40:27",
                                            "estimated": "2024-01-07T12:40:27"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T12:40:27",
                                            "estimated": "2024-01-07T12:40:27"
                                        }
                                    },
                                    "variant": {
                                        "key": "17-1-MH",
                                        "name": "Misericordia"
                                    },
                                    "bus": {
                                        "key": 341,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594735-71",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T13:09:27",
                                            "estimated": "2024-01-07T13:09:27"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T13:09:27",
                                            "estimated": "2024-01-07T13:09:27"
                                        }
                                    },
                                    "variant": {
                                        "key": "17-1-MH",
                                        "name": "Misericordia"
                                    },
                                    "bus": {
                                        "key": 883,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594736-63",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T13:38:27",
                                            "estimated": "2024-01-07T13:38:27"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T13:38:27",
                                            "estimated": "2024-01-07T13:38:27"
                                        }
                                    },
                                    "variant": {
                                        "key": "17-1-MH",
                                        "name": "Misericordia"
                                    },
                                    "bus": {
                                        "key": 313,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594732-71",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T14:07:27",
                                            "estimated": "2024-01-07T14:07:27"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T14:07:27",
                                            "estimated": "2024-01-07T14:07:27"
                                        }
                                    },
                                    "variant": {
                                        "key": "17-1-MH",
                                        "name": "Misericordia"
                                    },
                                    "bus": {
                                        "key": 140,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                }
                            ]
                        },
                        {
                            "route": {
                                "key": 18,
                                "number": 18,
                                "name": "North Main-Corydon",
                                "customer-type": "regular",
                                "coverage": "regular",
                                "badge-label": 18,
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
                                    "key": "25594430-52",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T12:33:00",
                                            "estimated": "2024-01-07T12:33:00"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T12:33:00",
                                            "estimated": "2024-01-07T12:33:00"
                                        }
                                    },
                                    "variant": {
                                        "key": "18-1-A",
                                        "name": "Assin Park"
                                    },
                                    "bus": {
                                        "key": 805,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594415-47",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T12:56:00",
                                            "estimated": "2024-01-07T12:56:00"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T12:56:00",
                                            "estimated": "2024-01-07T12:56:00"
                                        }
                                    },
                                    "variant": {
                                        "key": "18-1-A",
                                        "name": "Assin Park"
                                    },
                                    "bus": {
                                        "key": 447,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594431-52",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T13:18:00",
                                            "estimated": "2024-01-07T13:18:00"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T13:18:00",
                                            "estimated": "2024-01-07T13:18:00"
                                        }
                                    },
                                    "variant": {
                                        "key": "18-1-A",
                                        "name": "Assin Park"
                                    },
                                    "bus": {
                                        "key": 449,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594416-47",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T13:41:00",
                                            "estimated": "2024-01-07T13:41:00"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T13:41:00",
                                            "estimated": "2024-01-07T13:41:00"
                                        }
                                    },
                                    "variant": {
                                        "key": "18-1-A",
                                        "name": "Assin Park"
                                    },
                                    "bus": {
                                        "key": 452,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25594432-52",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T14:03:00",
                                            "estimated": "2024-01-07T14:03:00"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T14:03:00",
                                            "estimated": "2024-01-07T14:03:00"
                                        }
                                    },
                                    "variant": {
                                        "key": "18-1-A",
                                        "name": "Assin Park"
                                    },
                                    "bus": {
                                        "key": 730,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                }
                            ]
                        },
                        {
                            "route": {
                                "key": 60,
                                "number": 60,
                                "name": "Pembina",
                                "customer-type": "regular",
                                "coverage": "regular",
                                "badge-label": 60,
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
                                    "key": "25593599-6",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T12:25:33",
                                            "estimated": "2024-01-07T12:25:33"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T12:25:33",
                                            "estimated": "2024-01-07T12:25:33"
                                        }
                                    },
                                    "variant": {
                                        "key": "60-0-U",
                                        "name": "UofM"
                                    },
                                    "bus": {
                                        "key": 392,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25593600-6",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T12:57:33",
                                            "estimated": "2024-01-07T12:57:33"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T12:57:33",
                                            "estimated": "2024-01-07T12:57:33"
                                        }
                                    },
                                    "variant": {
                                        "key": "60-0-U",
                                        "name": "UofM"
                                    },
                                    "bus": {
                                        "key": 397,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25593601-6",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T13:28:33",
                                            "estimated": "2024-01-07T13:28:33"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T13:28:33",
                                            "estimated": "2024-01-07T13:28:33"
                                        }
                                    },
                                    "variant": {
                                        "key": "60-0-U",
                                        "name": "UofM"
                                    },
                                    "bus": {
                                        "key": 396,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25593602-6",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T14:00:33",
                                            "estimated": "2024-01-07T14:00:33"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T14:00:33",
                                            "estimated": "2024-01-07T14:00:33"
                                        }
                                    },
                                    "variant": {
                                        "key": "60-0-U",
                                        "name": "UofM"
                                    },
                                    "bus": {
                                        "key": 392,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                }
                            ]
                        },
                        {
                            "route": {
                                "key": 20,
                                "number": 20,
                                "name": "Academy-Watt",
                                "customer-type": "regular",
                                "coverage": "regular",
                                "badge-label": 20,
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
                                    "key": "25593366-38",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T12:45:27",
                                            "estimated": "2024-01-07T12:45:27"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T12:45:27",
                                            "estimated": "2024-01-07T12:45:27"
                                        }
                                    },
                                    "variant": {
                                        "key": "20-1-A",
                                        "name": "Airport"
                                    },
                                    "bus": {
                                        "key": 430,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25593367-38",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T13:29:27",
                                            "estimated": "2024-01-07T13:29:27"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T13:29:27",
                                            "estimated": "2024-01-07T13:29:27"
                                        }
                                    },
                                    "variant": {
                                        "key": "20-1-A",
                                        "name": "Airport"
                                    },
                                    "bus": {
                                        "key": 119,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                },
                                {
                                    "key": "25593368-38",
                                    "cancelled": "false",
                                    "times": {
                                        "arrival": {
                                            "scheduled": "2024-01-07T14:12:27",
                                            "estimated": "2024-01-07T14:12:27"
                                        },
                                        "departure": {
                                            "scheduled": "2024-01-07T14:12:27",
                                            "estimated": "2024-01-07T14:12:27"
                                        }
                                    },
                                    "variant": {
                                        "key": "20-1-A",
                                        "name": "Airport"
                                    },
                                    "bus": {
                                        "key": 199,
                                        "bike-rack": "false",
                                        "wifi": "false"
                                    }
                                }
                            ]
                        }
                    ]
                },
                "query-time": "2024-01-07T12:16:40"
            }
        )))
        .expect(1)
        .mount(&mock_winnipeg_transit_api)
        .await;

    let response = get(
        "/twilio?Body=10619",
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
    10619 WB Graham@Vaughan (The Bay)
    12:18p 16 St Vital Ctr
    12:19p BLUE Downtown
    12:22p BLUE Downtown
    12:25p 60 UofM
    12:33p 18 Assin Park
    "});
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
