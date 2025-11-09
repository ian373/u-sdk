//! DeepSeek sdk

mod types;
pub use types::*;

mod error;
pub use error::Error;

mod utils;

use async_stream::try_stream;
use bon::bon;
use bytes::{Buf, BytesMut};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use tokio_stream::{Stream, StreamExt};
use u_sdk_common::helper::{into_request_failed_error, parse_json_response};
use utils::check_msg_list;

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

    pub fn chat_builder(&self) -> ChatBuilder<'_> {
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
        check_msg_list(self.messages)?;

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

    pub async fn chat_by_stream(
        &self,
    ) -> Result<impl Stream<Item = Result<StreamEventData, Error>> + use<>, Error> {
        check_msg_list(&self.messages)?;

        if !self.stream {
            return Err(Error::Common(
                "Stream mode is not enabled. Use chat instead.".to_string(),
            ));
        }

        let resp = self
            .client
            .http_client
            .post(&format!("{}/chat/completions", BASE_URL))
            .json(self)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(into_request_failed_error(resp).await.into());
        }

        let mut byte_stream = resp.bytes_stream();

        let event_stream = try_stream! {
            let mut buffer = BytesMut::with_capacity(4096);

            while let Some(chunk) = byte_stream.next().await {
                // 如果底层网络错误，会通过 `?` 返回 Err(Error) 并终止流
                let chunk = chunk?;
                buffer.extend(chunk);

                // SSE 协议中，每条事件以 "\n\n" 分隔
                while let Some(pos) = buffer.windows(2).position(|w| w == b"\n\n") {
                    // 转成 &str，UTF-8 错误同样会返回 Err(Error)
                    let text = std::str::from_utf8(&buffer[..pos])
                        .map_err(|e| Error::Common(format!("Invalid UTF-8 sequence: {}", e)))?;
                    // 解析这一条事件，没有事件时（data: [DONE]）会返回 Ok(None)
                    if let Some(evt) = parse_event_block(text)? {
                        yield evt;
                    }
                    // 清除已处理的部分
                    buffer.advance(pos + 2);
                }
            }
        };

        Ok(Box::pin(event_stream))
    }
}

// 解析一段完整的 SSE 事件文本
fn parse_event_block(s: &str) -> Result<Option<StreamEventData>, Error> {
    let s = s.trim();
    // 结束标志
    if s.starts_with("data: [DONE]") {
        return Ok(None);
    }
    // 正常的数据行
    if let Some(rest) = s.strip_prefix("data:") {
        let json_str = rest.trim_start();
        let data: StreamEventData = serde_json::from_str(json_str)
            .map_err(|e| Error::Common(format!("Failed to parse stream event data: {}", e)))?;
        Ok(Some(data))
    } else {
        Err(Error::Common("Unknown event format".to_string()))
    }
}
//endregion
