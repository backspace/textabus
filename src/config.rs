// Adapted from https://dev.to/bdhobare/managing-application-config-in-rust-23ai
use std::collections::HashMap;
use url::Url;

#[derive(Clone, Debug)]
pub struct Config {
    pub admin_number: String,
    pub auth: String,
    pub database_url: Url,
    pub root_url: Url,
    pub textabus_number: String,
    pub twilio_account_sid: String,
    pub twilio_api_key_sid: String,
    pub twilio_api_key_secret: String,
    pub winnipeg_transit_api_key: String,
}

pub trait ConfigProvider {
    fn get_config(&self) -> &Config;
}

pub struct EnvVarProvider(Config);

impl EnvVarProvider {
    pub fn new(args: HashMap<String, String>) -> Self {
        let config = Config {
            admin_number: args
                .get("ADMIN_NUMBER")
                .expect("Missing admin number")
                .to_string(),
            auth: args.get("AUTH").expect("Missing auth").to_string(),
            database_url: Url::parse(args.get("DATABASE_URL").expect("Missing DATABASE_URL"))
                .expect("Unable to parse DATABASE_URL as a URL"),
            root_url: Url::parse(args.get("ROOT_URL").expect("Missing ROOT_URL"))
                .expect("Unable to parse ROOT_URL as a URL"),
            textabus_number: args
                .get("TEXTABUS_NUMBER")
                .expect("Missing textabus number")
                .to_string(),
            twilio_account_sid: args
                .get("TWILIO_ACCOUNT_SID")
                .expect("Missing Twilio account SID")
                .to_string(),
            twilio_api_key_sid: args
                .get("TWILIO_API_KEY_SID")
                .expect("Missing Twilio API key SID")
                .to_string(),
            twilio_api_key_secret: args
                .get("TWILIO_API_KEY_SECRET")
                .expect("Missing Twilio API key secret")
                .to_string(),
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
