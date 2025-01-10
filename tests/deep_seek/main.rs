use futures_util::stream::StreamExt;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub key: String,
}

fn get_deep_seek_key() -> String {
    let file_str = std::fs::read_to_string("./tests/deep_seek/config.toml").unwrap();
    let conf: Config = toml::from_str(&file_str).unwrap();
    conf.key
}

#[test]
fn get_key() {
    let token = get_deep_seek_key();
    println!("{}", token);
}

#[tokio::test]
async fn chat() {
    use tokio::time::{sleep, Duration};
    let token = get_deep_seek_key();
    let mut client = u_ali_sdk::deep_seek::DeepSeek::new(&token);
    let response = client.chat("介绍一下你自己").await;
    println!("{:#?}", response);
    sleep(Duration::from_secs(10)).await;
    let response = client.chat("介绍一下rust编程语言").await.unwrap();
    println!("{:#?}", response);

    let msg_list = client.get_msg_list();
    println!("{:#?}", msg_list);
}

#[tokio::test]
async fn chat_by_stream() {
    let token = get_deep_seek_key();
    let mut client = u_ali_sdk::deep_seek::DeepSeek::new(&token);
    let res = client.chat_by_stream("介绍一下你自己").await;
    match res {
        Ok(mut stream) => {
            while let Some(event) = stream.next().await {
                println!("{:?}", event);
            }
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}
