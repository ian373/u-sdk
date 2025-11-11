use crate::deep_seek::Client;
use bon::Builder;
use serde::{Deserialize, Serialize};

//region chat body
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
    // 下面两个未实现
    // pub(crate) tools: Option<()>,
    // #[serde(serialize_with = "serialize_tolls_choices")]
    // pub(crate) tool_choice: Option<()>,
    logprobs: Option<bool>,
    top_logprobs: Option<u32>, // Possible values: >= 0 and <= 20 指定此参数时，logprobs 必须为 true。
}

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
pub struct ResponseFormat<'a> {
    #[serde(rename = "type")]
    pub r#type: &'a str,
}

#[derive(Serialize, Debug)]
pub struct StreamOption {
    pub include_usage: bool,
}
//endregion

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
    // logprobs 未实现
    pub finish_reason: String,
    pub index: i32,
    pub message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
pub struct ChoiceMessage {
    // toll_calls没实现
    pub content: String,
    pub role: Role, // Role::Assistant
    // 仅适用于 deepseek-reasoner 模型。内容为 assistant 消息中在最终答案之前的推理内容。
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub completion_tokens: i32,
    pub prompt_tokens: i32,
    pub prompt_cache_hit_tokens: i32,
    pub prompt_cache_miss_tokens: i32,
    pub total_tokens: i32,
    pub completion_tokens_details: Option<CompletionTokensDetails>,
}

#[derive(Debug, Deserialize)]
pub struct CompletionTokensDetails {
    pub reasoning_tokens: i32,
}

#[test]
fn check_response_deserialize() {
    let json = r#"
{
  "id": "string", "choices": [
    {
      "finish_reason": "stop", "index": 0,
      "message": {
        "content": "string", "reasoning_content": "string",
        "tool_calls": [
          {
            "id": "string", "type": "function",
            "function": { "name": "string", "arguments": "string" }
          }
        ],
        "role": "assistant"
      },
      "logprobs": {
        "content": [
          {
            "token": "string", "logprob": 0, "bytes": [0],
            "top_logprobs": [
              { "token": "string", "logprob": 0, "bytes": [0] }
            ]
          }
        ]
      }
    }
  ],
  "created": 0, "model": "string", "system_fingerprint": "string",
  "object": "chat.completion",
  "usage": {
    "completion_tokens": 0, "prompt_tokens": 0,
    "prompt_cache_hit_tokens": 0, "prompt_cache_miss_tokens": 0,
    "total_tokens": 0,
    "completion_tokens_details": { "reasoning_tokens": 0 }
  }
}
"#;
    let response = serde_json::from_str::<ChatResponse>(json);
    assert!(
        response.is_ok(),
        "Failed to deserialize response: {:?}",
        response.err()
    );
}

//region stream event data
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
    pub reasoning_content: Option<String>,
    pub role: Option<Role>,
}

#[test]
fn check_stream_event_data_deserialize() {
    let json1 = r#"
{
  "id": "1f633d8bfc032625086f14113c411638",
  "choices": [
    {
      "index": 0,
      "delta": { "content": "", "role": "assistant" },
      "finish_reason": null,
      "logprobs": null
    }
  ],
  "created": 1718345013,
  "model": "deepseek-chat",
  "system_fingerprint": "fp_a49d71b8a1",
  "object": "chat.completion.chunk",
  "usage": null
}
"#;
    let json2 = r#"
{
  "choices": [
    {
      "delta": { "content": "", "role": null },
      "finish_reason": "stop",
      "index": 0,
      "logprobs": null
    }
  ],
  "created": 1718345013,
  "id": "1f633d8bfc032625086f14113c411638",
  "model": "deepseek-chat",
  "object": "chat.completion.chunk",
  "system_fingerprint": "fp_a49d71b8a1",
  "usage": {
    "completion_tokens": 9,
    "prompt_tokens": 17,
    "total_tokens": 26
  }
}
    "#;

    let response1 = serde_json::from_str::<StreamEventData>(json1);
    let response2 = serde_json::from_str::<StreamEventData>(json2);

    assert!(
        response1.is_ok(),
        "Failed to deserialize response1: {:?}",
        response1.err()
    );
    assert!(
        response2.is_ok(),
        "Failed to deserialize response2: {:?}",
        response2.err()
    );
}
//endregion

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
