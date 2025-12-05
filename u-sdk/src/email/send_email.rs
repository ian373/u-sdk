use super::Client;
use super::Error;
use super::utils::parse_json_response;
use bon::Builder;
use serde::{Deserialize, Serialize};
use u_sdk_common::helper::into_header_map;
use u_sdk_common::open_api_sign::{SignParams, get_openapi_request_header};

//region response
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SingleSendEmailResult {
    pub env_id: String,
    pub request_id: String,
}
//endregion

#[serde_with::skip_serializing_none]
#[derive(Builder, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SingleSendEmail<'a> {
    #[builder(start_fn)]
    #[serde(skip_serializing)]
    client: &'a Client,

    account_name: &'a str,
    address_type: &'a str,
    reply_to_address: &'a str,
    subject: &'a str,
    to_address: &'a str,
    click_trace: Option<&'a str>,
    from_alias: Option<&'a str>,
    html_body: Option<&'a str>,
    tag_name: Option<&'a str>,
    text_body: Option<&'a str>,
    reply_address: Option<&'a str>,
    reply_address_alias: Option<&'a str>,
}

impl Client {
    pub fn single_send_email(&self) -> SingleSendEmailBuilder<'_> {
        SingleSendEmail::builder(self)
    }
}

impl SingleSendEmail<'_> {
    pub async fn send(&self) -> Result<SingleSendEmailResult, Error> {
        // HtmlBody 和 TextBody 是针对不同类型的邮件内容，两者必须传其一
        if self.html_body.is_none() && self.text_body.is_none() {
            return Err(Error::Common(
                "one of html_body or text_body must be set".to_owned(),
            ));
        }

        let client = self.client;
        let creds = client.credentials_provider.load().await?;

        let sign_params = SignParams {
            req_method: "GET",
            host: &client.host,
            query_map: self,
            x_acs_action: "SingleSendMail",
            x_acs_version: "2015-11-23",
            x_acs_security_token: creds.sts_security_token.as_deref(),
            request_body: None,
            style: &client.style,
        };

        let (headers, url_) =
            get_openapi_request_header(&creds.access_key_secret, &creds.access_key_id, sign_params)
                .map_err(|e| Error::Common(format!("get_common_headers error: {}", e)))?;

        let resp = self
            .client
            .http_client
            .get(url_)
            .headers(into_header_map(headers))
            .send()
            .await?;

        let resp = parse_json_response(resp).await?;
        Ok(resp)
    }
}
