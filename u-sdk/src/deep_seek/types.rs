use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct Message {
    pub content: String,
    pub role: Role,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Serialize, Debug)]
pub(crate) struct ResponseFormat<'a> {
    #[serde(rename = "type")]
    pub(crate) r#type: &'a str,
}

#[derive(Serialize, Debug)]
pub(crate) struct StreamOption {
    pub(crate) include_usage: bool,
}

// chat response
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

#[derive(Debug)]
pub enum StreamEvent {
    Data(StreamEventData),
    Finish,
    Unknown(String),
}

#[derive(Deserialize, Debug)]
pub struct StreamEventData {
    pub id: String,
    pub choices: Vec<StreamDataChoices>,
    pub created: i64,
    pub model: String,
    pub system_fingerprint: String,
    pub object: String,
}

#[derive(Deserialize, Debug)]
pub struct StreamDataChoices {
    pub delta: Delta,
    pub finish_reason: Option<String>,
    pub index: i32,
}

#[derive(Deserialize, Debug)]
pub struct Delta {
    pub content: Option<String>,
    pub role: Option<Role>,
}

#[test]
fn check_stream_event_data_deserialize() {
    let json = r#"
{
  "id": "eff9fc88-c216-46ec-928f-64d8af234eee",
  "object": "chat.completion.chunk",
  "created": 1736350935,
  "model": "deepseek-chat",
  "system_fingerprint": "fp_3a5770e1b4",
  "choices": [
    {
      "index": 0,
      "delta": {
        "role": "assistant",
        "content": ""
      },
      "logprobs": null,
      "finish_reason": null
    }
  ]
}
"#;
    let data: StreamEventData = serde_json::from_str(json).unwrap();
    println!("{:#?}", data);
}

#[derive(Deserialize, Debug)]
pub struct CheckBalanceResponse {
    pub is_available: bool,
    pub balance_infos: Vec<BalanceInfo>,
}

#[derive(Deserialize, Debug)]
pub struct BalanceInfo {
    pub currency: String,
    pub total_balance: String,
    pub granted_balance: String,
    pub topped_up_balance: String,
}
