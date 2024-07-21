use serde::Deserialize;
use u_ali_sdk::server_chan::*;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub token: String,
}

impl Config {
    pub fn get_conf() -> Self {
        let file_str = std::fs::read_to_string("tests/server_chan/config.toml").unwrap();
        let conf = toml::from_str(&file_str).unwrap();

        conf
    }
}

#[tokio::test]
async fn server_chan_test() {
    let conf = Config::get_conf();
    let server_chan = ServerChan::new(&conf.token);
    let params = SendParams {
        title: "test--title",
        description: Some("this is a description"),
        is_no_ip: true,
        ..Default::default()
    };
    server_chan.send_msg(params).await.unwrap();
}
