pub const SINGLE_SEND_EMAIL_BASE_URL: &str = "https://dm.aliyuncs.com";

pub struct SingleSendEmail {
    pub account_name: String,
    pub address_type: u8,
    pub reply_to_address: String,
    pub subject: String,
    pub to_address: String,
    pub action: Option<String>,
    pub click_trace: Option<u8>,
    pub from_alias: Option<String>,
    pub html_body: Option<String>,
    pub tag_name: Option<String>,
    pub text_body: Option<String>,
    pub reply_address: Option<String>,
    pub reply_address_alias: Option<String>,
}

pub struct SingleSendEmailSuccessResponse {
    pub env_id: String,
    pub request_id: String,
}
