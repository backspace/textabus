// Adapted from https://dev.to/bdhobare/managing-application-config-in-rust-23ai
use std::collections::HashMap;
use url::Url;

#[derive(Clone, Debug)]
pub struct Config {
    pub auth: String,
    pub database_url: Url,
    pub winnipeg_transit_api_key: String,
}

pub trait ConfigProvider {
    fn get_config(&self) -> &Config;
}

pub struct EnvVarProvider(Config);

impl EnvVarProvider {
    pub fn new(args: HashMap<String, String>) -> Self {
        let config = Config {
            auth: args.get("AUTH").expect("Missing auth").to_string(),
            database_url: Url::parse(args.get("DATABASE_URL").expect("Missing DATABASE_URL"))
                .expect("Unable to parse DATABASE_URL as a URL"),
            winnipeg_transit_api_key: args
                .get("WINNIPEG_TRANSIT_API_KEY")
                .expect("Missing WINNIPEG_TRANSIT_API_KEY")
                .to_string(),
        };

        EnvVarProvider(config)
    }
}

impl ConfigProvider for EnvVarProvider {
    fn get_config(&self) -> &Config {
        &self.0
    }
}

impl Default for EnvVarProvider {
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}
