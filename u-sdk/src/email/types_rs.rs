use serde::Deserialize;

//region desc_account_summary
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DescAccountSummaryResult {
    pub daily_quota: u32,
    pub domains: u32,
    pub enable_times: u32,
    pub mail_addresses: u32,
    pub max_quota_level: u32,
    pub month_quota: u32,
    pub quota_level: u8,
    pub request_id: String,
    pub tags: u32,
    pub templates: u32,
    pub user_status: u8,
}
//endregion

//region query_domain_by_param
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct QueryDomainByParamResult {
    pub page_number: u32,
    pub page_size: u32,
    pub request_id: String,
    pub total_count: u32,
    #[serde(rename = "data")]
    pub data: Data,
}

#[derive(Deserialize, Debug)]
pub struct Data {
    pub domain: Vec<PerInfo>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PerInfo {
    pub cname_auth_status: u8,
    pub confirm_status: u8,
    pub create_time: String,
    pub domain_id: u32,
    pub domain_name: String,
    pub domain_status: u8,
    pub icp_status: u8,
    pub mx_auth_status: u8,
    pub spf_auth_status: u8,
    pub utc_create_time: u64,
    pub domain_record: String,
}
//endregion

//region get_ip_protection
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct GetIpProtectionResult {
    pub ip_protection: String,
    pub request_id: String,
}
//endregion
