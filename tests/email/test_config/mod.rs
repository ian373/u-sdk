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
        let file_str = std::fs::read_to_string("tests/email/test_config/config.toml").unwrap();
        let conf = toml::from_str(&file_str).unwrap();

        conf
    }
}
