use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use std::env;
use textabus::{app, InjectableServices};
use tokio::net::TcpListener;
use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

use textabus::config::{ConfigProvider, EnvVarProvider};

struct TestApp {
    pub address: String,
}

pub async fn get(
    path: &str,
    mut services: InjectableServices,
) -> Result<reqwest::Response, reqwest::Error> {
    services = set_up_services(services).await;

    let app_address = spawn_app(services).await.address;

    let client = Client::new();
    let url = format!("{}{}", app_address, path);

    client.get(&url).send().await
}

#[allow(dead_code)]
pub async fn get_with_auth(
    path: &str,
    mut services: InjectableServices,
) -> Result<reqwest::Response, reqwest::Error> {
    let env_config_provider = EnvVarProvider::new(env::vars().collect());
    let config = &env_config_provider.get_config();

    services = set_up_services(services).await;

    let app_address = spawn_app(services).await.address;

    let client = Client::new();
    let url = format!("{}{}", app_address, path);

    client
        .get(&url)
        .header(
            "Authorization",
            format!(
                "Basic {}",
                general_purpose::STANDARD.encode(config.auth.clone())
            ),
        )
        .send()
        .await
}

async fn set_up_services(mut services: InjectableServices) -> InjectableServices {
    if services.winnipeg_transit_api_address.is_none() {
        let mock_winnipeg_transit_api = MockServer::start().await;

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(0)
            .named("Mock Winnipeg Transit API")
            .mount(&mock_winnipeg_transit_api)
            .await;

        services = InjectableServices {
            db: services.db,
            winnipeg_transit_api_address: Some("http://localhost:1313".to_string()),
        };
    }

    services
}

async fn spawn_app(services: InjectableServices) -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    tokio::spawn(async move {
        axum::serve(
            listener,
            app(services)
                .await
                .into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });

    TestApp { address }
}
