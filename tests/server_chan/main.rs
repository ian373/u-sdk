use serde::Deserialize;
use toml::Table;
use u_sdk::server_chan::*;

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

fn get_server_chan3_conf() -> (i32, String) {
    let file_str = std::fs::read_to_string("tests/server_chan/config.toml").unwrap();
    let conf = toml::from_str::<Table>(&file_str).unwrap();
    let uid = conf
        .get("server_chan3_uid")
        .unwrap()
        .as_integer()
        .expect("uid must be an integer") as i32;
    let key = conf
        .get("server_chan3_send_key")
        .expect("server_chan3_send_key must be set")
        // 这里需要as_str，如果不是，会返回`"xxx"`，包含引号
        .as_str()
        .expect("server_chan3_send_key must be a string");

    (uid, key.to_owned())
}

#[tokio::test]
async fn server_chan_turbo_test() {
    let conf = Config::get_conf();
    let server_chan = ServerChanTurbo::new(&conf.token);
    let params = TurboSendParams {
        title: "test--title",
        description: Some("this is a description"),
        is_no_ip: true,
        ..Default::default()
    };
    server_chan.send_msg_turbo(params).await.unwrap();
}

#[tokio::test]
async fn server_chan3_test() {
    let (uid, key) = get_server_chan3_conf();
    let server_chan = ServerChan3::new(uid, &key);
    let params = SendParams {
        title: "test--title",
        desp: Some("this is a description"),
        short: Some("short"),
        tags: Some("test"),
    };
    let res = server_chan.send_msg(&params).await;
    if let Err(e) = res {
        println!("error: {:?}", e);
    }
}
