//! Server酱3 sdk

use bon::{Builder, bon};
use reqwest::StatusCode;
use serde::Serialize;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("request failed: code: {code}\nbody: {body}")]
    API { code: StatusCode, body: String },
    #[error("use reqwest error:\n {0}")]
    Reqwest(#[from] reqwest::Error),
}

#[derive(Builder, Serialize)]
pub struct SendMsg<'a> {
    #[builder(start_fn)]
    #[serde(skip_serializing)]
    client: &'a Client,
    /// 标签列表，多个标签使用竖线`|`分隔
    // 注意这个#[builder(filed)字段有顺序要求，需要放在start_fn之后，finish_fn之前
    #[builder(field)]
    #[serde(
        serialize_with = "serialize_tags",
        skip_serializing_if = "Vec::is_empty"
    )]
    tags: Vec<&'a str>,
    /// 推送的标题
    title: &'a str,
    #[serde(rename = "desp", skip_serializing_if = "Option::is_none")]
    /// 推送的正文内容，则为必填，支持markdown
    description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// 推送消息的简短描述，用于指定消息卡片的内容部分，尤其是在推送markdown的时候
    short: Option<&'a str>,
}

fn serialize_tags<S>(tags: &[&str], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let tags_str = tags.join("|");
    serializer.serialize_str(&tags_str)
}

impl<'a, S: send_msg_builder::State> SendMsgBuilder<'a, S> {
    pub fn tag(mut self, tag: &'a str) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn tags(mut self, tags: impl IntoIterator<Item = &'a str>) -> Self {
        self.tags.extend(tags.into_iter());
        self
    }
}

/// 使用Server酱3
pub struct Client {
    url: String,
    http_client: reqwest::Client,
}

#[bon]
impl Client {
    #[builder]
    pub fn new(uid: i32, key: &str) -> Self {
        Self {
            url: format!("https://{}.push.ft07.com/send/{}.send", uid, key),
            http_client: reqwest::Client::new(),
        }
    }

    pub fn send_msg(&self) -> SendMsgBuilder<'_> {
        SendMsg::builder(self)
    }
}

impl SendMsg<'_> {
    pub async fn send(&self) -> Result<(), Error> {
        let client = self.client;
        let resp = client
            .http_client
            .post(&client.url)
            .json(self)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(Error::API {
                code: resp.status(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(())
    }
}
