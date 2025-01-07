use reqwest::StatusCode;
use serde::{Deserialize, Serialize, Serializer};

const BASE_URL: &str = "https://api.deepseek.com";

#[derive(Serialize, Debug)]
pub struct Message {
    content: String,
    role: Role,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Serialize, Debug)]
enum Model {
    #[serde(rename = "deepseek-chat")]
    DeepSeekChat,
}

#[derive(Serialize, Debug)]
struct ResponseFormat {
    #[serde(rename = "type")]
    r#type: ResponseFormatType,
}

#[derive(Serialize, Debug)]
enum ResponseFormatType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "json_object")]
    JsonObject,
}

#[derive(Serialize, Debug)]
struct RequestParams {
    messages: Vec<Message>,
    model: Model,
    frequency_penalty: Option<f32>, //Default 0.0 Possible values: >= -2 and <= 2
    max_tokens: Option<i32>,        //Default 4096 Possible values: > 1
    presence_penalty: Option<f32>,  //Default 0.0 Possible values: >= -2 and <= 2
    response_format: Option<ResponseFormat>, //Default text
    stop: Option<()>,
    stream: bool,
    stream_options: Option<()>,
    temperature: Option<f32>, //Default 1.0 Possible values: >= 0 and <= 2
    top_p: Option<f32>,       //Default 1.0 Possible values: <= 1
    tools: Option<()>,
    #[serde(serialize_with = "serialize_tolls_choices")]
    tool_choice: Option<()>,
    logprobs: bool,
    top_logprobs: Option<i32>, //Possible values: >= 0 and <= 20 指定此参数时，logprobs 必须为 true。
}

impl Default for RequestParams {
    fn default() -> Self {
        Self {
            messages: vec![Message {
                content: "You are a helpful assistant".to_string(),
                role: Role::System,
            }],
            model: Model::DeepSeekChat,
            frequency_penalty: Some(0.0),
            max_tokens: Some(4096),
            presence_penalty: Some(0.0),
            response_format: Some(ResponseFormat {
                r#type: ResponseFormatType::Text,
            }),
            stop: None,
            stream: false,
            stream_options: None,
            temperature: Some(1.0),
            top_p: Some(1.0),
            tools: None,
            tool_choice: None,
            logprobs: false,
            top_logprobs: None,
        }
    }
}

fn serialize_tolls_choices<S>(value: &Option<()>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(v) => serializer.serialize_some(v), // 如果是 Some，正常序列化
        None => serializer.serialize_some("none"), // 如果是 None，序列化为 "none"
    }
}

#[test]
fn serialize_request_params() {
    let json = serde_json::to_string_pretty(&RequestParams::default()).unwrap();
    println!("{}", json);
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub created: i64,
    pub model: String,
    pub system_fingerprint: String,
    pub object: String,
    pub usage: Usage,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub finish_reason: String,
    pub index: i32,
    pub message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
pub struct ChoiceMessage {
    pub content: String,
    pub role: Role, // Role::Assistant
                    // toll_calls先不用
                    // logprobs先不用
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub completion_tokens: i32,
    pub prompt_tokens: i32,
    pub prompt_cache_hit_tokens: i32,
    pub prompt_cache_miss_tokens: i32,
    pub total_tokens: i32,
}

pub struct DeepSeek {
    api_key: String,
    client: reqwest::Client,
    request_params: RequestParams,
}

#[test]
fn check_response_deserialize() {
    let json = r#"
{
  "id":"string","choices":[{
    "finish_reason":"stop","index":0,"message":{
      "content":"string","tool_calls":[{
        "id":"string","type":"function","function":{
          "name":"string","arguments":"string"
        }
      }],"role":"assistant"
    },"logprobs":{
      "content":[{
        "token":"string","logprob":0,"bytes":[0],"top_logprobs":[{
          "token":"string","logprob":0,"bytes":[0]
        }]
      }]
    }
  }],"created":0,"model":"string","system_fingerprint":"string",
  "object":"chat.completion","usage":{
    "completion_tokens":0,"prompt_tokens":0,"prompt_cache_hit_tokens":0,
    "prompt_cache_miss_tokens":0,"total_tokens":0
  }
}
"#;
    let response: ChatResponse = serde_json::from_str(json).unwrap();
    println!("{:#?}", response);
}

impl DeepSeek {
    pub fn new(api_key: &str) -> Self {
        let request_params = RequestParams::default();
        Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
            request_params,
        }
    }

    pub async fn chat(&mut self, msg: &str) -> Result<ChatResponse, String> {
        self.request_params.messages.push(Message {
            content: msg.to_string(),
            role: Role::User,
        });

        // todo
        if self.request_params.stream {
            eprintln!("stream is not supported yet");
            return Err("stream is not supported yet".to_string());
        }

        let response = self
            .client
            .post(&format!("{}/chat/completions", BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&self.request_params)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        match response.status() {
            StatusCode::OK => {
                let response = response
                    .json::<ChatResponse>()
                    .await
                    .map_err(|e| e.to_string())?;
                self.request_params.messages.push(Message {
                    content: response.choices[0].message.content.clone(),
                    role: response.choices[0].message.role.clone(),
                });
                Ok(response)
            }
            status => Err(format!("Request failed with status: {}", status)),
        }
    }

    pub fn get_msg_list(&self) -> &[Message] {
        &self.request_params.messages
    }
}
