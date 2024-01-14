use textabus::{app, InjectableServices};
use tokio::net::TcpListener;
use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

struct TestApp {
    pub address: String,
}

pub async fn get(
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
            db: services.db,
            winnipeg_transit_api_address: Some("http://localhost:1313".to_string()),
        };
    }

    let app_address = spawn_app(services).await.address;

    let client = reqwest::Client::new();
    let url = format!("{}{}", app_address, path);

    client.get(&url).send().await
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
