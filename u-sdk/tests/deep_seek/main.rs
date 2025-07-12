use futures_util::StreamExt;
use serde::Deserialize;
use u_sdk::deep_seek::Client;
use u_sdk::deep_seek::{Message, Role, StreamEvent};

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
async fn chat_test() {
    let token = get_deep_seek_key();
    let client = Client::builder().api_key(token).build();
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
    let resp = client
        .chat_builder()
        .messages(&mut msg_list)
        .model("deepseek-chat")
        .build()
        .chat()
        .await;

    match resp {
        Ok(response) => println!("Response: {:#?}", response),
        Err(e) => println!("Error: {}", e),
    }
}

#[tokio::test]
async fn chat_by_stream_test() {
    let token = get_deep_seek_key();
    let client = Client::builder().api_key(token).build();
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
        .chat_builder()
        .stream(true)
        .messages(&msg_list)
        .model("deepseek-chat")
        .build()
        .chat_by_stream()
        .await
        .expect("Failed to create stream");
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
    let client = Client::builder().api_key(token).build();
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
