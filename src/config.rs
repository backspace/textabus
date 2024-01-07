// Adapted from https://dev.to/bdhobare/managing-application-config-in-rust-23ai
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Config {
    pub winnipeg_transit_api_key: String,
}

pub trait ConfigProvider {
    fn get_config(&self) -> &Config;
}

pub struct EnvVarProvider(Config);

impl EnvVarProvider {
    pub fn new(args: HashMap<String, String>) -> Self {
        let config = Config {
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
