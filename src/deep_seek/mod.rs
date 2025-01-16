pub mod types;

use crate::deep_seek::types::{FixedParams, Role};
use async_stream::stream;
use futures_util::{stream::StreamExt, Stream};
use reqwest::StatusCode;
use types::{ChatResponse, Message, RequestParams, StreamEvent, StreamEventData};

const BASE_URL: &str = "https://api.deepseek.com";

pub struct DeepSeek {
    api_key: String,
    client: reqwest::Client,
    fixed_params: FixedParams,
}

impl DeepSeek {
    pub fn new(api_key: &str) -> Self {
        let fixed_params = FixedParams::default();
        Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
            fixed_params,
        }
    }

    pub async fn chat(&mut self, msg_list: &[Message]) -> Result<ChatResponse, String> {
        check_msg_list(msg_list)?;

        // 防止 stream 为 true
        if self.fixed_params.stream {
            self.fixed_params.stream = false;
        }

        let request_params = RequestParams {
            messages: msg_list,
            fix_params: &self.fixed_params,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_params)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        match response.status() {
            StatusCode::OK => {
                let response = response
                    .json::<ChatResponse>()
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(response)
            }
            status => Err(format!("Request failed with status: {}", status)),
        }
    }

    pub async fn chat_by_stream(
        &mut self,
        msg_list: &[Message],
    ) -> Result<impl Stream<Item = StreamEvent>, String> {
        check_msg_list(msg_list)?;

        // 打开stream功能
        self.fixed_params.stream = true;

        let request_params = RequestParams {
            messages: msg_list,
            fix_params: &self.fixed_params,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_params)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.headers().contains_key("content-type")
            || !response.headers()["content-type"]
                .to_str()
                .unwrap_or("")
                .starts_with("text/event-stream")
        {
            return Err("Expected content-type: text/event-stream".to_string());
        }

        let mut body = response.bytes_stream();
        let mut buffer = String::new();

        let s = stream! {
            'req_stream: while let Some(chunk) = body.next().await {
                let chunk = chunk.unwrap();
                buffer.push_str(&String::from_utf8_lossy(&chunk));
                while let Some(pos) = buffer.find("\n\n") {
                    let event_data = &buffer[..pos];
                    // println!("{}", event_data);

                    if event_data.starts_with("data: [DONE]") {
                        yield StreamEvent::Finish;
                        continue 'req_stream;
                    }

                    if let Some(event) = event_data.strip_prefix("data:") {
                        let event_data =serde_json::from_str::<StreamEventData>(event);
                        match event_data {
                            Ok(data) => {
                                yield StreamEvent::Data(data);
                            }
                            Err(_) => {
                                yield StreamEvent::Unknown(event.to_string());
                            }
                        }
                    } else {
                        yield StreamEvent::Unknown(event_data.to_string());
                    }

                    buffer = buffer[pos + 2..].to_string();
                }
            }
        };

        Ok(Box::pin(s))
    }
}

fn check_msg_list(msg_list: &[Message]) -> Result<(), String> {
    if msg_list.is_empty() {
        return Err("msg_list is empty".to_string());
    } else if msg_list[0].role != Role::Assistant {
        return Err("The first message role must be Assistant".to_string());
    } else if msg_list.last().unwrap().role != Role::User {
        return Err("The last message role must be User".to_string());
    }

    Ok(())
}
