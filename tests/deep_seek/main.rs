use futures_util::StreamExt;
use serde::Deserialize;
use u_ali_sdk::deep_seek::types::{Role, StreamEvent};

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
    use u_ali_sdk::deep_seek::types::Message;

    let token = get_deep_seek_key();
    let mut client = u_ali_sdk::deep_seek::DeepSeek::new(&token);
    let mut msg_list = vec![
        Message {
            content: "You are a helpful assistant.".to_string(),
            role: Role::Assistant,
        },
        Message {
            content: "What is rust programming language?".to_string(),
            role: Role::User,
        },
    ];
    let response = client.chat(&msg_list).await;
    match response {
        Ok(mut resp) => {
            let answer = resp.choices.pop().unwrap();
            msg_list.push(Message {
                content: answer.message.content,
                role: answer.message.role,
            });
            println!("{:#?}", msg_list);
        }
        Err(e) => {
            println!("{:#?}", e);
        }
    }
}

#[tokio::test]
async fn chat_by_stream() {
    use u_ali_sdk::deep_seek::types::Message;

    let token = get_deep_seek_key();
    let mut client = u_ali_sdk::deep_seek::DeepSeek::new(&token);
    let msg_list = vec![
        Message {
            content: "You are a helpful assistant.".to_string(),
            role: Role::Assistant,
        },
        Message {
            content: "What is rust programming language?".to_string(),
            role: Role::User,
        },
    ];
    let mut stream = client
        .chat_by_stream(&msg_list)
        .await
        .expect("chat_by_stream error");
    let mut s = String::new();
    while let Some(event) = stream.next().await {
        println!("{:#?}", event);
        match event {
            StreamEvent::Data(mut data) => {
                let answer = data.choices.pop().unwrap();
                s.push_str(&answer.delta.content.unwrap_or("".to_string()));
            }
            StreamEvent::Unknown(u) => {
                println!("unknown:\n{}", u);
            }
            _ => (),
        }
    }

    println!("=============\nresult: \n{}", s);
}

#[tokio::test]
async fn check_balance_test() {
    let token = get_deep_seek_key();
    let client = u_ali_sdk::deep_seek::DeepSeek::new(&token);
    let balance = client.check_balance().await;
    match balance {
        Ok(b) => {
            println!("{:#?}", b);
        }
        Err(e) => {
            println!("{:#?}", e);
        }
    }
}
