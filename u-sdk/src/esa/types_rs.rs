use super::Client;
use bon::Builder;
use serde::de::IntoDeserializer;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;

//region ListSites request
#[serde_with::skip_serializing_none]
#[derive(Builder, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListSites<'a> {
    #[builder(start_fn)]
    #[serde(skip_serializing)]
    pub(crate) client: &'a Client,

    /// 标签过滤规则。
    #[builder(field)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tag_filter: Vec<TagFilterItem>,

    /// 站点名称。用于查询的过滤条件。
    site_name: Option<String>,

    /// 站点名称的搜索匹配模式。默认为精确匹配。
    site_search_type: Option<SiteSearchType>,

    /// 页码。默认值：1。
    page_number: Option<i32>,

    /// 分页大小。默认值：500。
    page_size: Option<i32>,

    /// 资源组 ID。用于查询的过滤条件。
    resource_group_id: Option<String>,

    /// 站点状态。用于查询的过滤条件。
    status: Option<String>,

    /// 仅企业版，传 true 时代表仅查询企业版的站点。
    only_enterprise: Option<bool>,

    /// 套餐订阅类型。
    plan_subscribe_type: Option<PlanSubscribeType>,

    /// 加速区域。
    coverage: Option<Coverage>,

    /// 接入类型。
    access_type: Option<AccessType>,

    /// 排序字段，默认按照创建时间排序。
    order_by: Option<OrderBy>,
}

impl<S: list_sites_builder::State> ListSitesBuilder<'_, S> {
    /// 添加标签过滤规则
    pub fn tag_filter(mut self, item: TagFilterItem) -> Self {
        self.tag_filter.push(item);
        self
    }

    /// 批量添加标签过滤规则
    pub fn tag_filters(mut self, items: impl IntoIterator<Item = TagFilterItem>) -> Self {
        self.tag_filter.extend(items);
        self
    }
}

/// TagFilter 数组里的 object
#[derive(Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct TagFilterItem {
    /// 标签键，用于查询的过滤条件。
    pub key: Option<String>,
    /// 标签值，用于查询的过滤条件。
    pub value: Option<String>,
}

/// 站点名称搜索匹配模式：prefix/suffix/exact/fuzzy
#[derive(Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SiteSearchType {
    Prefix,
    Suffix,
    Exact,
    Fuzzy,
}

/// 套餐订阅类型：basicplan/standardplan/advancedplan/enterpriseplan
#[derive(Serialize, Clone)]
pub enum PlanSubscribeType {
    #[serde(rename = "basicplan")]
    Basic,
    #[serde(rename = "standardplan")]
    Standard,
    #[serde(rename = "advancedplan")]
    Advanced,
    #[serde(rename = "enterpriseplan")]
    Enterprise,
}

/// 加速区域：domestic/global/overseas
#[derive(Serialize, Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Coverage {
    Domestic,
    Global,
    Overseas,
}

/// 接入类型：NS/CNAME（注意：API 值大小写敏感时建议按原样）
#[derive(Serialize, Clone, Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AccessType {
    NS,
    CNAME,
}

/// 排序字段：gmtCreate/visitTime
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum OrderBy {
    GmtCreate,
    VisitTime,
}
//endregion

//region ListSites response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListSitesResponse {
    /// 请求 ID
    pub request_id: String,

    /// 返回数据的页码
    pub page_number: i32,

    /// 每页显示的站点个数
    pub page_size: i32,

    /// 总站点数量
    pub total_count: i32,

    /// 查询到的站点信息列表
    pub sites: Vec<SiteInfo>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SiteInfo {
    /// 站点接入类型
    pub access_type: AccessType,

    /// 站点的 CNAME 后缀
    pub cname_zone: String,

    /// 站点加速区域
    pub coverage: Coverage,

    /// 站点创建时间（ISO8601, UTC）
    #[serde(with = "time::serde::iso8601")]
    pub create_time: OffsetDateTime,

    /// 站点更新时间（ISO8601, UTC）
    #[serde(with = "time::serde::iso8601")]
    pub update_time: OffsetDateTime,

    /// 站点绑定的套餐实例 ID
    pub instance_id: String,

    /// 站点分配的 NS 列表，逗号分隔
    pub name_server_list: String,

    /// 套餐名称
    pub plan_name: String,

    /// 套餐规格名称
    pub plan_spec_name: Option<String>,

    /// 资源组 ID
    pub resource_group_id: String,

    /// 站点 ID
    pub site_id: i64,

    /// 站点名称
    pub site_name: String,

    /// 站点状态
    pub status: SiteStatus,

    /// 站点标签（Key-Value 形式）
    pub tags: Option<HashMap<String, String>>,

    /// 站点归属校验码（CNAME 接入时使用）
    pub verify_code: String,

    /// 站点访问时间（ISO8601, UTC）
    #[serde(with = "time::serde::iso8601")]
    pub visit_time: OffsetDateTime,

    /// 站点停用原因
    #[serde(deserialize_with = "de_option_empty_string_as_none")]
    pub offline_reason: Option<OfflineReason>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SiteStatus {
    Pending,
    Active,
    Offline,
    Moved,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OfflineReason {
    ExpirationArrears,
    InternallyDisabled,
    MissingIcp,
    ContentViolation,
    ProactivelyDisabled,
}

// offline reason这个字段如果实例没有offline，那么值会为空字符串`""`而不是不出现在字段中，需要特殊处理
// 这个方法处理这种情况
pub fn de_option_empty_string_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    // 先按 Option<serde_json::Value> 或 Option<String> 都行；
    // 用 Option<String> 的好处是明确只处理字符串场景
    // 处理字段不存在的情况
    let opt = Option::<String>::deserialize(deserializer)?;

    match opt {
        None => Ok(None),
        // 处理字段存在但值为空字符串的情况
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => {
            // 关键点：把 String 再交给 T 的 Deserialize（复用 serde 对 enum 的规则）
            T::deserialize(s.into_deserializer()).map(Some)
        }
    }
}
//endregion
