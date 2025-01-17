use crate::error::Error;
use serde::Serialize;

#[derive(Default)]
pub struct TurboSendParams<'a> {
    pub title: &'a str,
    /// 消息内容
    pub description: Option<&'a str>,
    pub short: Option<&'a str>,
    pub is_no_ip: bool,
    /// 最多支持两个通道，如果输入的通道超过两个，只取前两个
    pub channel: Option<&'a [u8]>,
}

/// 使用Server酱Turbo
pub struct ServerChanTurbo<'a> {
    token: &'a str,
    client: reqwest::Client,
}

impl<'a, 'b> ServerChanTurbo<'a>
where
    'a: 'b,
{
    pub fn new(token: &'a str) -> Self {
        Self {
            token,
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_msg_turbo(&self, params: TurboSendParams<'b>) -> Result<(), Error> {
        if params.title.is_empty() {
            return Err(Error::AnyError("title can't be empty.".to_string()));
        }

        let mut p = Vec::with_capacity(5);
        p.push(("title", params.title));
        if let Some(description) = params.description {
            p.push(("desp", description));
        }
        if let Some(short) = params.short {
            p.push(("short", short));
        }
        if params.is_no_ip {
            p.push(("noip", "1"));
        }

        let channel_str;
        if let Some(channel) = params.channel {
            let mut iter = channel.iter();
            channel_str = match (iter.next(), iter.next()) {
                (Some(c1), None) => c1.to_string(),
                (Some(c1), Some(c2)) => format!("{}|{}", c1, c2),
                _ => {
                    return Err(Error::AnyError(
                        "The number of channels can only be 1 or 2".to_string(),
                    ))
                }
            };
            p.push(("channel", &channel_str));
        }

        let url = format!("https://sctapi.ftqq.com/{}.send", self.token);
        self.client.post(url).form(&p).send().await?;

        Ok(())
    }
}

#[derive(Serialize)]
pub struct SendParams<'a> {
    /// 推送的标题
    pub title: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// 推送的正文内容，则为必填，支持markdown
    pub desp: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// 标签列表，多个标签使用竖线`|`分隔
    pub tags: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// 推送消息的简短描述，用于指定消息卡片的内容部分，尤其是在推送markdown的时候
    pub short: Option<&'a str>,
}

/// 使用Server酱3
pub struct ServerChan3 {
    url: String,
    client: reqwest::Client,
}

impl ServerChan3 {
    pub fn new(uid: i32, key: &str) -> Self {
        Self {
            url: format!("https://{}.push.ft07.com/send/{}.send", uid, key),
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_msg(&self, params: &SendParams<'_>) -> Result<(), Error> {
        let resp = self
            .client
            .post(&self.url)
            .json(&params)
            .send()
            .await
            .map_err(|_| Error::AnyError("\"reqwest\" send error".to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(Error::AnyError(format!(
                "Request failed with status: {}\n{}",
                resp.status(),
                resp.text()
                    .await
                    .unwrap_or("Failed to get response text".to_string())
            )))
        }
    }
}
