use super::Client;
use super::parse_json_response;
use super::{Error, OPENAPI_STYLE, OPENAPI_VERSION};
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use time::OffsetDateTime;
use u_sdk_common::helper::into_header_map;
use u_sdk_common::open_api_sign::{SignParams, get_openapi_request_header};

impl Client {
    pub fn describe_prefix_list_attributes(&self) -> DescribePrefixListAttributesBuilder<'_> {
        DescribePrefixListAttributes::builder(self)
    }

    pub fn modify_prefix_list(&self) -> ModifyPrefixListBuilder<'_> {
        ModifyPrefixList::builder(self)
    }
}

/// [DescribePrefixListAttributes](https://help.aliyun.com/zh/ecs/developer-reference/api-ecs-2014-05-26-describeprefixlistattributes)
#[derive(Builder, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DescribePrefixListAttributes<'a> {
    #[serde(skip_serializing)]
    #[builder(start_fn)]
    pub(crate) client: &'a Client,
    /// 地域 ID。您可以调用 DescribeRegions 查看最新的阿里云地域列表。
    region_id: &'a str,
    /// 前缀列表 ID
    prefix_list_id: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DescribePrefixListAttributesResponse {
    #[serde(with = "time::serde::iso8601")]
    pub creation_time: OffsetDateTime,
    pub max_entries: i32,
    pub request_id: String,
    pub description: String,
    pub address_family: AddressFamily,
    pub prefix_list_name: String,
    pub prefix_list_id: String,
    pub entries: Option<EntriesWrapper>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EntriesWrapper {
    pub entry: Vec<PrefixListEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PrefixListEntry {
    pub description: String,
    pub cidr: String,
}

#[derive(Debug, Deserialize)]
pub enum AddressFamily {
    IPv4,
    IPv6,
}

impl DescribePrefixListAttributes<'_> {
    pub async fn send(&self) -> Result<DescribePrefixListAttributesResponse, Error> {
        let client = self.client;
        let creds = client.credentials_provider.load().await?;

        let sign_params = SignParams {
            req_method: "GET",
            host: &client.host,
            query_map: self,
            x_acs_action: "DescribePrefixListAttributes",
            x_acs_version: OPENAPI_VERSION,
            x_acs_security_token: creds.sts_security_token.as_deref(),
            request_body: None,
            style: &OPENAPI_STYLE,
        };

        let (common_headers, url_) =
            get_openapi_request_header(&creds.access_key_secret, &creds.access_key_id, sign_params)
                .map_err(|e| {
                    Error::Common(format!("failed to get openapi request header: {}", e))
                })?;
        let header_map = into_header_map(common_headers);

        let resp = client
            .http_client
            .get(url_)
            .headers(header_map)
            .send()
            .await?;

        let data = parse_json_response(resp).await?;
        Ok(data)
    }
}

/// [ModifyPrefixList](https://help.aliyun.com/zh/ecs/developer-reference/api-ecs-2014-05-26-modifyprefixlist)
#[serde_with::skip_serializing_none]
#[derive(Serialize, Builder)]
#[serde(rename_all = "PascalCase")]
pub struct ModifyPrefixList<'a> {
    #[builder(start_fn)]
    #[serde(skip_serializing)]
    pub(crate) client: &'a Client,

    #[builder(field)]
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "serialize_add_entry"
    )]
    add_entry: Vec<(Cow<'a, str>, Option<&'a str>)>,
    #[builder(field)]
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "serialize_remove_entry"
    )]
    remove_entry: Vec<Cow<'a, str>>,

    region_id: &'a str,
    prefix_list_id: &'a str,
    prefix_list_name: Option<&'a str>,
    description: Option<&'a str>,
}

impl<'a, S: modify_prefix_list_builder::State> ModifyPrefixListBuilder<'a, S> {
    pub fn add_entry(
        mut self,
        cidr: impl Into<Cow<'a, str>>,
        description: Option<&'a str>,
    ) -> Self {
        self.add_entry.push((cidr.into(), description));
        self
    }

    /// 如果cidr为None，则不添加该条目，此时description参数会被忽略
    pub fn maybe_add_entry(
        mut self,
        cidr: Option<impl Into<Cow<'a, str>>>,
        description: Option<&'a str>,
    ) -> Self {
        if let Some(cidr) = cidr {
            self.add_entry.push((cidr.into(), description));
        }
        self
    }

    pub fn add_entries<I, C>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = (C, Option<&'a str>)>,
        C: Into<Cow<'a, str>>,
    {
        self.add_entry.extend(
            items
                .into_iter()
                .map(|(cidr, description)| (cidr.into(), description)),
        );
        self
    }

    pub fn maybe_add_entries<I, C>(mut self, items: Option<I>) -> Self
    where
        I: IntoIterator<Item = (C, Option<&'a str>)>,
        C: Into<Cow<'a, str>>,
    {
        if let Some(items) = items {
            self.add_entry.extend(
                items
                    .into_iter()
                    .map(|(cidr, description)| (cidr.into(), description)),
            );
        }
        self
    }

    pub fn remove_entry(mut self, cidr: impl Into<Cow<'a, str>>) -> Self {
        self.remove_entry.push(cidr.into());
        self
    }

    pub fn maybe_remove_entry(mut self, cidr: Option<impl Into<Cow<'a, str>>>) -> Self {
        if let Some(cidr) = cidr {
            self.remove_entry.push(cidr.into());
        }
        self
    }

    pub fn remove_entries<I, C>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<Cow<'a, str>>,
    {
        self.remove_entry.extend(items.into_iter().map(Into::into));
        self
    }

    pub fn maybe_remove_entries<I, C>(mut self, items: Option<I>) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<Cow<'a, str>>,
    {
        if let Some(items) = items {
            self.remove_entry.extend(items.into_iter().map(Into::into));
        }
        self
    }
}

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct AddEntryItem<'a> {
    cidr: Cow<'a, str>,
    description: Option<&'a str>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct RemoveEntryItem<'a> {
    cidr: Cow<'a, str>,
}

fn serialize_add_entry<S>(
    items: &[(Cow<str>, Option<&str>)],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let add_entry_items: Vec<AddEntryItem> = items
        .iter()
        .map(|(cidr, description)| AddEntryItem {
            cidr: Cow::Borrowed(cidr.as_ref()),
            description: *description,
        })
        .collect();
    add_entry_items.serialize(serializer)
}

fn serialize_remove_entry<S>(items: &[Cow<str>], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let remove_entry_items: Vec<RemoveEntryItem> = items
        .iter()
        .map(|cidr| RemoveEntryItem {
            cidr: Cow::Borrowed(cidr.as_ref()),
        })
        .collect();
    remove_entry_items.serialize(serializer)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ModifyPrefixListResponse {
    pub request_id: String,
}

impl ModifyPrefixList<'_> {
    pub async fn send(&self) -> Result<ModifyPrefixListResponse, Error> {
        let client = self.client;
        let creds = client.credentials_provider.load().await?;

        let sign_params = SignParams {
            req_method: "POST",
            host: &client.host,
            query_map: self,
            x_acs_action: "ModifyPrefixList",
            x_acs_version: OPENAPI_VERSION,
            x_acs_security_token: creds.sts_security_token.as_deref(),
            request_body: None,
            style: &OPENAPI_STYLE,
        };

        let (common_headers, url_) =
            get_openapi_request_header(&creds.access_key_secret, &creds.access_key_id, sign_params)
                .map_err(|e| {
                    Error::Common(format!("failed to get openapi request header: {}", e))
                })?;
        let header_map = into_header_map(common_headers);

        let resp = client
            .http_client
            .post(url_)
            .headers(header_map)
            .send()
            .await?;

        let data = parse_json_response::<ModifyPrefixListResponse>(resp).await?;
        Ok(data)
    }
}
