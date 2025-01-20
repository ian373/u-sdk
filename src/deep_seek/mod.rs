pub mod types;

use crate::deep_seek::types::{CheckBalanceResponse, FixedParams, Role};
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

    /// 多轮对话形式
    ///
    /// 发送的形式：
    ///
    /// ```json
    /// // 第一条可以是prompt
    /// {"content": "You are a helpful assistant", "role": "system" }
    /// {"content": "Hi", "role": "user" }
    ///
    /// // 或者直接是user
    /// {"content": "Hi", "role": "user" }
    /// ```
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

        let response = self.send(request_params).await?;
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

        let response = self.send(request_params).await?;
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

    async fn send(&self, params: RequestParams<'_>) -> Result<reqwest::Response, String> {
        let response = self
            .client
            .post(format!("{}/chat/completions", BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Ok(response)
    }

    pub async fn check_balance(&self) -> Result<CheckBalanceResponse, String> {
        let response = self
            .client
            .get(format!("{}/user/balance", BASE_URL))
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        match response.status() {
            StatusCode::OK => {
                let response = response
                    .json::<CheckBalanceResponse>()
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(response)
            }
            status => Err(format!("Request failed with status: {}", status)),
        }
    }
}

fn check_msg_list(msg_list: &[Message]) -> Result<(), String> {
    // 多轮对话的形式

    if msg_list.is_empty() {
        return Err("msg_list is empty".to_string());
    } else if msg_list.last().unwrap().role != Role::User {
        // 最后一条消息必须是 User
        return Err("The last message role must be User".to_string());
    }

    Ok(())
}
