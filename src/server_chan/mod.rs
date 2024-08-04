use crate::error::Error;

#[derive(Default)]
pub struct SendParams<'a> {
    pub title: &'a str,
    /// 消息内容
    pub description: Option<&'a str>,
    pub short: Option<&'a str>,
    pub is_no_ip: bool,
    /// 最多支持两个通道，如果输入的通道超过两个，只取前两个
    pub channel: Option<&'a [u8]>,
}

pub struct ServerChan<'a> {
    token: &'a str,
    client: reqwest::Client,
}

impl<'a, 'b> ServerChan<'a>
where
    'a: 'b,
{
    pub fn new(token: &'a str) -> Self {
        Self {
            token,
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_msg(&self, params: SendParams<'b>) -> Result<(), Error> {
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
