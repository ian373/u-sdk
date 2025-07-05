use serde::Deserialize;
use u_sdk::server_chan::*;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub uid: i32,
    pub key: String,
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
    let client = Client::builder().uid(conf.uid).key(&conf.key).build();

    let resp = client
        .send_msg()
        .title("test--title")
        .description("this is a description")
        .short("short")
        .tag("123")
        .tags(["tag1", "tag2"])
        .build()
        .send()
        .await;
    if let Err(e) = resp {
        eprintln!("Error sending message: {}", e);
    } else {
        println!("Message sent successfully");
    }
}
