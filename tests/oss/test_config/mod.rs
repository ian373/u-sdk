use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AliConfig {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub endpoint: String,
    pub bucket_name: String,
}

impl AliConfig {
    pub fn get_conf() -> Self {
        let file_str = std::fs::read_to_string("tests/oss/test_config/config.toml").unwrap();
        let conf = toml::from_str(&file_str).unwrap();

        conf
    }
}
