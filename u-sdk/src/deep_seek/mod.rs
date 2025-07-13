mod types;
pub use types::*;

mod error;
mod utils;
pub use error::Error;

use async_stream::stream;
use bon::{Builder, bon};
use common_lib::helper::{into_request_failed_error, parse_json_response};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::Serialize;
use tokio_stream::{Stream, StreamExt};

const BASE_URL: &str = "https://api.deepseek.com";

//region client
pub struct Client {
    http_client: reqwest::Client,
}

#[bon]
impl Client {
    #[builder(on(String, into))]
    pub fn new(api_key: String) -> Self {
        let mut header_map = HeaderMap::new();
        let mut auth_val = HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap();
        auth_val.set_sensitive(true);
        header_map.insert(AUTHORIZATION, auth_val);

        let http_client = reqwest::Client::builder()
            .default_headers(header_map)
            .build()
            .unwrap();

        Self { http_client }
    }

    pub fn chat_builder(&self) -> ChatBuilder {
        Chat::builder(self)
    }

    pub async fn check_balance(&self) -> Result<CheckBalanceResponse, Error> {
        let resp = self
            .http_client
            .get(format!("{}/user/balance", BASE_URL))
            .send()
            .await?;

        let res = parse_json_response(resp).await?;
        Ok(res)
    }
}
//endregion

//region chat
#[derive(Builder, Serialize)]
pub struct Chat<'a> {
    #[builder(start_fn)]
    #[serde(skip_serializing)]
    pub(crate) client: &'a Client,
    #[builder(field)]
    stop: Vec<String>,
    pub(crate) messages: &'a [Message],
    model: &'a str,
    frequency_penalty: Option<f32>, // Default 0.0 Possible values: >= -2 and <= 2
    max_tokens: Option<u32>,        // Default 4096 Possible values: > 1
    presence_penalty: Option<f32>,  // Default 0.0 Possible values: >= -2 and <= 2
    response_format: Option<ResponseFormat<'a>>, // Default text
    #[builder(default = false)]
    pub(crate) stream: bool,
    stream_options: Option<StreamOption>,
    temperature: Option<f32>, // Default 1.0 Possible values: >= 0 and <= 2
    top_p: Option<f32>,       // Default 1.0 Possible values: <= 1
    // pub(crate) tools: Option<()>,
    // #[serde(serialize_with = "serialize_tolls_choices")]
    // pub(crate) tool_choice: Option<()>,
    #[builder(default = false)]
    logprobs: bool,
    top_logprobs: Option<i32>, // Possible values: >= 0 and <= 20 指定此参数时，logprobs 必须为 true。
}

impl Chat<'_> {
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
    pub async fn chat(&self) -> Result<ChatResponse, Error> {
        utils::check_msg_list(self.messages)?;

        // 防止 stream 为 true
        if self.stream {
            return Err(Error::Common(
                "Stream mode is enabled. Use chat_by_stream instead.".to_string(),
            ));
        }

        let client = self.client;
        let resp = client
            .http_client
            .post(format!("{}/chat/completions", BASE_URL))
            .json(self)
            .send()
            .await?;

        let res = parse_json_response(resp).await?;
        Ok(res)
    }

    pub async fn chat_by_stream(&self) -> Result<impl Stream<Item = StreamEvent> + use<>, Error> {
        utils::check_msg_list(self.messages)?;

        if !self.stream {
            return Err(Error::Common(
                "Stream mode is not enabled. Use chat instead.".to_string(),
            ));
        }

        let client = self.client;
        let resp = client
            .http_client
            .post(format!("{}/chat/completions", BASE_URL))
            .json(self)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(into_request_failed_error(resp).await.into());
        }

        let mut body = resp.bytes_stream();
        let mut buffer = String::new();

        // REFACTOR 需要优化，应该是buf为字节，然后判断b"\n\n"这种
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
//endregion
