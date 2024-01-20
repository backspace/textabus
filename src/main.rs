use sqlx::PgPool;
use std::env;
use tokio::net::TcpListener;

use textabus::{
    app,
    config::{ConfigProvider, EnvVarProvider},
    InjectableServices,
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let env_config_provider = EnvVarProvider::new(env::vars().collect());
    let config = &env_config_provider.get_config();

    let database_url = &config.database_url;
    let db = PgPool::connect(database_url.as_str()).await.unwrap();

    sqlx::migrate!()
        .run(&db)
        .await
        .expect("Failed to run migrations");

    let listener_address = "0.0.0.0:1312";
    let listener = TcpListener::bind(listener_address)
        .await
        .expect("Failed to bind port 1312");

    println!("textabus listening on port 1312");

    axum::serve(
        listener,
        app(InjectableServices {
            db,
            twilio_address: Some("https://api.twilio.com".to_string()),
            winnipeg_transit_api_address: Some("https://api.winnipegtransit.com".to_string()),
        })
        .await
        .into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();
}
