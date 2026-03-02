use super::utils::{
    de_non_empty_object, de_option_empty_string_as_none, parse_json_response, se_as_json_string,
};
use super::{Error, OPENAPI_STYLE, OPENAPI_VERSION};
use crate::esa::Client;
use bon::Builder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;
use u_sdk_common::helper::into_header_map;
use u_sdk_common::open_api_sign::{SignParams, get_openapi_request_header};

impl Client {
    pub fn list_records(&self) -> ListRecordsBuilder<'_> {
        ListRecords::builder(self)
    }

    pub fn get_record(&self) -> GetRecordBuilder<'_> {
        GetRecord::builder(self)
    }

    pub fn create_record(&self) -> CreateRecordBuilder<'_> {
        CreateRecord::builder(self)
    }
}

//region ListRecords request
/// [ListRecords](https://help.aliyun.com/zh/edge-security-acceleration/esa/api-esa-2024-09-10-listrecords)
#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Builder)]
#[serde(rename_all = "PascalCase")]
pub struct ListRecords<'a> {
    #[builder(start_fn)]
    #[serde(skip_serializing)]
    pub(crate) client: &'a Client,

    /// 站点 ID，可通过调用 ListSites 接口获取
    site_id: i64,

    /// 记录名称，用于查询的过滤条件
    record_name: Option<String>,

    /// 记录名称的搜索匹配模式
    /// 默认为精确匹配，取值：
    /// prefix：前缀匹配
    /// suffix：后缀匹配
    /// exact：精确匹配
    /// fuzzy：模糊匹配
    record_match_type: Option<RecordMatchType>,

    /// 页码，默认值为 1
    page_number: Option<i32>,

    /// 分页大小，默认值为 500
    page_size: Option<i32>,

    /// 记录的源站类型，用于查询的过滤条件
    /// 仅 CNAME 记录可以通过该字段筛选
    source_type: Option<SourceType>,

    /// 记录加速时的业务场景，用于查询的过滤条件
    /// 取值：image_video、api、web
    biz_name: Option<BizName>,

    /// 记录是否开启代理加速，用于查询的过滤条件
    proxied: Option<bool>,

    /// 记录的 DNS 记录类型，用于查询的过滤条件
    record_type: Option<DnsRecordType>,
}

/// 记录名称的搜索匹配模式
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordMatchType {
    Prefix, // 前缀匹配
    Suffix, // 后缀匹配
    Exact,  // 精确匹配
    Fuzzy,  // 模糊匹配
}

/// 记录的源站类型
#[derive(Debug, Serialize, Deserialize)]
pub enum SourceType {
    OSS,    // OSS 源站
    S3,     // S3 源站
    LB,     // 负载均衡器源站
    OP,     // 源地址池源站
    Domain, // 普通域名源站
}

/// 记录加速时的业务场景
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BizName {
    ImageVideo, // 视频图片
    Api,        // API 接口
    Web,        // Web 网页
}

/// DNS 记录类型
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DnsRecordType {
    #[serde(rename = "A/AAAA")]
    AOrAAAA, // A 或 AAAA 记录，自动选择 IPv4 或 IPv6 地址
    CNAME,  // CNAME 记录，域名的别名
    MX,     // MX 记录，邮件交换记录
    TXT,    // TXT 记录，文本记录
    NS,     // NS 记录，域名服务器记录
    SRV,    // SRV 记录，服务定位记录
    CAA,    // CAA 记录，证书颁发机构授权记录
    CERT,   // CERT 记录，证书记录
    SMIMEA, // SMIMEA 记录，S/MIME 证书记录
    SSHFP,  // SSHFP 记录，SSH 公钥指纹记录
    TLSA,   // TLSA 记录，TLS 认证记录
    URI,    // URI 记录，统一资源标识记录
}

/// 请求返回参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListRecordsResponse {
    /// 请求 ID
    pub request_id: String,

    /// 当前页码
    pub page_number: i32,

    /// 每页显示的记录条数
    pub page_size: i32,

    /// 记录总条数
    pub total_count: i32,

    /// DNS 记录信息
    pub records: Vec<DnsRecord>,
}

/// 单个 DNS 记录信息
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DnsRecord {
    /// 记录加速时的业务场景
    #[serde(deserialize_with = "de_option_empty_string_as_none")]
    pub biz_name: Option<BizName>,

    /// 记录的具体 DNS 信息
    pub data: DnsData,

    /// 记录的创建时间
    #[serde(with = "time::serde::rfc3339")]
    pub create_time: OffsetDateTime,

    /// 记录的更新时间
    #[serde(with = "time::serde::rfc3339")]
    pub update_time: OffsetDateTime,

    /// 记录是否开启代理加速
    pub proxied: bool,

    /// 记录 ID
    pub record_id: i64,

    /// 记录的源站类型
    #[serde(deserialize_with = "de_option_empty_string_as_none")]
    pub record_source_type: Option<SourceType>,

    /// 记录名称
    pub record_name: String,

    /// 记录的DNS类型
    pub record_type: DnsRecordType,

    /// 记录所属站点的 ID
    pub site_id: i64,

    /// 记录所属站点的名称
    pub site_name: String,

    /// 记录的过期时间
    pub ttl: i32,

    /// 记录的 CNAME
    pub record_cname: String,

    /// 记录的备注
    pub comment: String,

    /// CNAME 记录的源站鉴权信息
    // 在 ListRecords 接口中，鉴权信息在没有时会返回空对象 `{}`;
    // 在 GetRecord 接口中，鉴权信息在没有时不存在该字段。使用 `de_non_empty_object` + default 处理这两种情况
    #[serde(default, deserialize_with = "de_non_empty_object")]
    pub auth_conf: Option<AuthConf>,

    /// 回源 HOST 策略
    #[serde(deserialize_with = "de_option_empty_string_as_none")]
    pub host_policy: Option<HostPolicy>,
}

/// 记录的具体 DNS 信息
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DnsData {
    /// 记录值或部分内容
    pub value: String,

    /// 记录的优先级
    pub priority: Option<i32>,

    /// 记录的标志位
    pub flag: Option<i32>,

    /// 标签
    pub tags: Option<HashMap<String, String>>,

    /// 权重
    pub weight: Option<i32>,

    /// 端口
    pub port: Option<i32>,

    /// 证书类型
    pub r#type: Option<i32>,

    /// 公钥标识
    pub key_tag: Option<i32>,

    /// 加密算法
    pub algorithm: Option<i32>,

    /// 证书信息
    pub certificate: Option<String>,

    /// 用途标识
    pub usage: Option<i32>,

    /// 证书选择器
    pub selector: Option<i32>,

    /// 证书匹配类型
    pub matching_type: Option<i32>,

    /// 公钥指纹值
    pub fingerprint: Option<String>,

    /// 标签
    pub tag: Option<String>,
}

/// CNAME 记录的源站鉴权信息
#[derive(Debug, Deserialize, Serialize)]
pub struct AuthConf {
    /// 鉴权类型
    pub auth_type: Option<AuthType>,

    /// 访问密钥
    pub access_key: Option<String>,

    /// 秘密访问密钥
    pub secret_key: Option<String>,

    /// 签名版本
    pub version: Option<String>,

    /// 区域
    pub region: Option<String>,
}

/// 鉴权类型
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    Public,              // 公共读
    Private,             // 私有读
    PrivateSameAccount,  // 私有同账号读
    PrivateCrossAccount, // 私有跨账号读
}

/// 回源 HOST 策略
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HostPolicy {
    FollowHostname,     // 跟随请求 HOST
    FollowOriginDomain, // 跟随源站域名
}

impl ListRecords<'_> {
    pub async fn send(&self) -> Result<ListRecordsResponse, Error> {
        let client = self.client;
        let creds = client.credentials_provider.load().await?;

        let sign_params = SignParams {
            req_method: "GET",
            host: &client.host,
            query_map: self,
            x_acs_action: "ListRecords",
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
//endregion

// region GetRecord request
/// [GetRecord](https://help.aliyun.com/zh/edge-security-acceleration/esa/api-esa-2024-09-10-getrecord)
#[derive(Builder, Debug)]
pub struct GetRecord<'a> {
    #[builder(start_fn)]
    pub(crate) client: &'a Client,
    pub(crate) record_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetRecordResponse {
    pub request_id: String,
    pub record_model: DnsRecord,
}

impl GetRecord<'_> {
    pub async fn send(&self) -> Result<GetRecordResponse, Error> {
        let client = self.client;
        let creds = client.credentials_provider.load().await?;

        let sign_params = SignParams {
            req_method: "GET",
            host: &client.host,
            query_map: HashMap::from([("RecordId", self.record_id)]),
            x_acs_action: "GetRecord",
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
// endregion

// region CreateRecord
/// [CreateRecord](https://help.aliyun.com/zh/edge-security-acceleration/esa/api-esa-2024-09-10-createrecord)
#[serde_with::skip_serializing_none]
#[derive(Builder, Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateRecord<'a> {
    #[serde(skip_serializing)]
    #[builder(start_fn)]
    pub(crate) client: &'a Client,

    site_id: i64,
    record_name: &'a str,

    /// 只有 CNAME 或 A/AAAA 记录可开启；true/false
    proxied: Option<bool>,

    /// DNS 记录类型：A/AAAA、CNAME、TXT 等
    r#type: DnsRecordType,

    /// 仅 CNAME 添加时要求填写；不传默认 Domain
    source_type: Option<SourceType>,

    /// proxied=true 时必填；proxied=false 时不需要
    biz_name: Option<BizName>,

    ttl: u32,

    #[serde(serialize_with = "se_as_json_string")]
    data: RecordData<'a>,

    /// 备注，最大 100 字符
    comment: Option<&'a str>,

    /// CNAME 源站鉴权信息（主要用于 SourceType=OSS/S3 等场景）
    auth_conf: Option<AuthConf>,

    /// 回源 HOST 策略，仅 CNAME 生效
    host_policy: Option<HostPolicy>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Builder)]
#[serde(rename_all = "PascalCase")]
pub struct RecordData<'a> {
    /// A/AAAA, CNAME, NS, MX, TXT, CAA, SRV, URI 时必填（含义随 type 变化）
    value: Option<&'a str>,

    /// MX, SRV, URI 必填；0~65535；越小优先级越高
    priority: Option<u16>,

    /// CAA 必填；0~255
    flag: Option<u8>,

    /// CAA 必填；issue/issuewild/iodef
    tag: Option<CaaTag>,

    /// SRV, URI 必填；0~65535
    weight: Option<u16>,

    /// SRV 必填；0~65535
    port: Option<u16>,

    /// CERT/SSHFP 必填（文档称 Type=integer，但示例可能是 RSA 等；这里用 String 表示更稳妥）
    r#type: Option<&'a str>,

    /// CERT 必填；0~65535
    key_tag: Option<u16>,

    /// CERT/SMIMEA/TLSA 必填（base64 或证书内容）
    certificate: Option<&'a str>,

    /// SMIMEA/TLSA 必填；0~255
    usage: Option<u8>,

    /// SMIMEA/TLSA 必填；0~255
    selector: Option<u8>,

    /// SMIMEA/TLSA 必填；0~255
    matching_type: Option<u8>,

    /// SSHFP 必填
    fingerprint: Option<&'a str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CaaTag {
    Issue,
    IssueWild,
    Iodef,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateRecordResponse {
    pub request_id: String,
    pub record_id: i64,
}

impl CreateRecord<'_> {
    pub async fn send(&self) -> Result<CreateRecordResponse, Error> {
        let client = self.client;
        let creds = client.credentials_provider.load().await?;

        let sign_params = SignParams {
            req_method: "POST",
            host: &client.host,
            query_map: self,
            x_acs_action: "CreateRecord",
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

        let data = parse_json_response(resp).await?;
        Ok(data)
    }
}
// endregion
