use config::Config;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AliConfig {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub account_name: String,
    pub to_address: String,
}

impl AliConfig {
    pub fn get_conf() -> Self {
        let conf = Config::builder()
            .add_source(config::File::with_name("tests/test_config/config.toml"))
            .build()
            .unwrap()
            .try_deserialize::<Self>()
            .unwrap();

        conf
    }
}
