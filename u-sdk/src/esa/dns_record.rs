use super::utils::{de_option_empty_string_as_none, parse_json_response};
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
    A,    // A 记录，映射域名到 IP 地址
    AAAA, // AAAA 记录，IPv6 地址记录
    #[serde(rename = "A/AAAA")]
    AOrAAAA, // A 或 AAAA 记录，自动选择 IPv4 或 IPv6 地址
    CNAME, // CNAME 记录，域名的别名
    MX,   // MX 记录，邮件交换记录
    TXT,  // TXT 记录，文本记录
    NS,   // NS 记录，域名服务器记录
    PTR,  // PTR 记录，反向 DNS 记录
    SOA,  // SOA 记录，授权记录
    SRV,  // SRV 记录，服务定位记录
    CAA,  // CAA 记录，证书颁发机构授权记录
    NAPTR, // NAPTR 记录，命名授权指针
    LOC,  // LOC 记录，地理位置信息记录
    HINFO, // HINFO 记录，主机信息记录
    RP,   // RP 记录，责任人记录
    TKEY, // TKEY 记录，事务密钥记录
    TSIG, // TSIG 记录，事务签名记录
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
    pub auth_conf: AuthConf,

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
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    Public,              // 公共读
    Private,             // 私有读
    PrivateSameAccount,  // 私有同账号读
    PrivateCrossAccount, // 私有跨账号读
}

/// 回源 HOST 策略
#[derive(Debug, Deserialize)]
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
