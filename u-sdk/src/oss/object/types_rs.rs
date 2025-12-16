use super::super::Client;
use crate::oss::Error;
use crate::oss::utils::validate_object_name;
use bon::Builder;
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{Value, json};
use serde_with::{DisplayFromStr, serde_as};
use std::collections::HashMap;
use std::path::Path;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc2822;

// region:    --- pub object
/// Header字段中：
/// - content_md5: 由程序自动添加
/// - content_length：由程序自动添加
/// - e_tag：不添加
#[serde_with::skip_serializing_none]
#[derive(Builder, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PutObject<'a> {
    #[builder(start_fn)]
    #[serde(skip_serializing)]
    pub(crate) client: &'a Client,
    // x-meta-* Header，不序列化，收集到map中以供访问
    #[builder(field)]
    #[serde(skip_serializing)]
    pub(crate) custom_metas: HashMap<String, String>,

    // region 公共请求头
    // Authorization 自动添加
    /// 对于MIME不会进行检查合法性检查
    content_type: Option<&'a str>,
    // content_length  自动添加
    // date 自动添加
    // host 自动添加
    // x_oss_security_token  未支持
    // endregion

    // region api请求头
    cache_control: Option<&'a str>,
    content_disposition: Option<&'a str>,
    content_encoding: Option<&'a str>,
    // content_md5  自动添加
    expires: Option<&'a str>,
    x_oss_forbid_overwrite: Option<&'a str>,
    x_oss_server_side_encryption: Option<&'a str>,
    x_oss_server_side_data_encryption: Option<&'a str>,
    x_oss_server_side_encryption_key_id: Option<&'a str>,
    x_oss_object_acl: Option<&'a str>,
    x_oss_storage_class: Option<&'a str>,
    // x-oss-meta-*  将由custom_metas转换为`x-oss-meta-key: value`形式添加
    x_oss_tagging: Option<&'a str>,
    // endregion

    // callback
    #[serde(skip_serializing)]
    pub(crate) callback: Option<OssCallBack>,
}

pub trait OssMetaExt<'a>: Sized {
    /// 需要让实现者返回对内部 custom_metas 的可变引用
    fn custom_metas_mut(&mut self) -> &mut HashMap<String, String>;

    fn x_meta(mut self, key: &'a str, val: &'a str) -> Self {
        self.custom_metas_mut()
            .insert(format!("x-oss-meta-{key}"), val.to_owned());
        self
    }

    fn x_metas(mut self, metas: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        for (key, val) in metas {
            self.custom_metas_mut()
                .insert(format!("x-oss-meta-{key}"), val.to_owned());
        }
        self
    }
}

// x-oss-meta-* Header<br/>
// 对于`XOtherHeader`中的key: value，会自动转换为: `x-oss-meta-key: value`，并添加到请求的Header
impl<'a, S: put_object_builder::State> OssMetaExt<'a> for PutObjectBuilder<'a, S> {
    fn custom_metas_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.custom_metas
    }
}

pub enum PutObjectBody<'a> {
    Bytes(Vec<u8>),
    FilePath(&'a Path),
}

#[derive(Debug)]
pub struct PutObjectResponseHeader {
    pub content_md5: String,
    pub x_oss_hash_crc64ecma: String,
    pub x_oss_version_id: Option<String>,
}
// endregion: --- pub object

// region post object
/// 此方法用于预签名生成给前端使用，如果需要使用sdk上传使用[PutObject]方法
///
/// 注意：上传的签名在有效期内都可以使用，要考虑恶意多次上传等等；所以限制条件要写好
#[derive(Builder)]
pub struct PostObject<'a> {
    #[builder(start_fn)]
    pub(crate) client: &'a Client,
    // PostObject API 表单元素的x-meta-*
    #[builder(field)]
    pub(crate) custom_metas: HashMap<String, (String, String)>,

    // POST v4签名表单元素
    pub(crate) bucket: Option<String>,
    // x_oss_security_token: 自动添加
    // (min, max)
    pub(crate) content_length_range: Option<(i32, i32)>,
    pub(crate) success_action_status: Option<(String, String)>,
    /// 前端请求的时候表单必须要有key字段，如果这里为None，就意味着前端的key字段可以是任意值，没有限制
    pub(crate) key: Option<(String, String)>,
    pub(crate) content_type: Option<(String, Vec<String>)>,
    pub(crate) cache_control: Option<(String, Vec<String>)>,

    // PostObject API 表单元素
    pub(crate) content_disposition: Option<(String, String)>,
    pub(crate) content_encoding: Option<String>,
    pub(crate) expires: Option<OffsetDateTime>,
    // policy是前端添加的必带字段，由后端生成并传给前端
    // policy: Option<String>,
    pub(crate) x_oss_server_side_data_encryption: Option<String>,
    pub(crate) x_oss_server_side_encryption_key_id: Option<String>,
    pub(crate) x_oss_content_type: Option<String>,
    pub(crate) x_oss_forbid_overwrite: Option<String>,
    pub(crate) x_oss_object_acl: Option<String>,
    pub(crate) x_oss_storage_class: Option<String>,
    pub(crate) success_action_redirect: Option<(String, String)>,
    // x-oss-meta-*，由于bon的顺序要求放到了前面
    // file

    // callback: https://help.aliyun.com/zh/oss/developer-reference/callback
    pub(crate) callback: Option<OssCallBack>,
}

impl<'a, S: post_object_builder::State> PostObjectBuilder<'a, S> {
    fn custom_metas_mut(&mut self) -> &mut HashMap<String, (String, String)> {
        &mut self.custom_metas
    }

    /// 添加`x-oss-meta-{key}: val`，其中val是一个元组，表示操作符和值，例如：("eq", "value")
    ///
    /// `key`不需要包含`x-oss-meta-`前缀，函数会自动添加
    pub fn x_meta(mut self, key: &'a str, val: (&'a str, &'a str)) -> Self {
        self.custom_metas_mut().insert(
            format!("x-oss-meta-{key}"),
            (val.0.to_owned(), val.1.to_owned()),
        );
        self
    }

    /// 批量添加`x-oss-meta-{key}: val`
    pub fn x_metas(
        mut self,
        metas: impl IntoIterator<Item = (&'a str, (&'a str, &'a str))>,
    ) -> Self {
        for (key, val) in metas {
            self.custom_metas_mut().insert(
                format!("x-oss-meta-{key}"),
                (val.0.to_owned(), val.1.to_owned()),
            );
        }
        self
    }
}

#[derive(Serialize)]
pub(crate) struct PostPolicy<'a> {
    pub expiration: String,
    #[serde(serialize_with = "serialize_conditions")]
    pub conditions: PostPolicyCondition<'a>,
}

// 不用担心转义问题，serde_json会自动处理
// 通过[PostObject]的builder收集所有数据，包括POST API的表单元素，POST V4签名所需的表单元素，callback参数
// 然后序列化为policy conditions。通过这样，就可以强制要求前端上传时必须携带这些字段，并且符合要求
// 如果不全部集中到这里来做限制，前端可以随意填写表单域，导致上传不符合要求
pub(crate) struct PostPolicyCondition<'a> {
    // POST v4签名表单元素（字段）
    pub(crate) bucket: Option<String>,
    // 固定为`OSS4-HMAC-SHA256`，自动添加
    pub(crate) x_oss_signature_version: String,
    pub(crate) x_oss_credential: String,
    pub(crate) x_oss_security_token: Option<String>,
    pub(crate) x_oss_date: String,
    // (min, max)
    pub(crate) content_length_range: Option<(i32, i32)>,
    pub(crate) success_action_status: Option<(String, String)>,
    pub(crate) key: Option<(String, String)>,
    pub(crate) content_type: Option<(String, Vec<String>)>,
    pub(crate) cache_control: Option<(String, Vec<String>)>,

    // PostObject API 表单元素（字段）
    pub(crate) content_disposition: Option<(String, String)>,
    pub(crate) content_encoding: Option<String>,
    pub(crate) expires: Option<OffsetDateTime>,
    pub(crate) x_oss_server_side_data_encryption: Option<String>,
    pub(crate) x_oss_server_side_encryption_key_id: Option<String>,
    pub(crate) x_oss_content_type: Option<String>,
    pub(crate) x_oss_forbid_overwrite: Option<String>,
    pub(crate) x_oss_object_acl: Option<String>,
    pub(crate) x_oss_storage_class: Option<String>,
    pub(crate) success_action_redirect: Option<(String, String)>,
    pub(crate) custom_metas: HashMap<String, (String, String)>,

    // callback
    pub(crate) callback_b64: Option<&'a str>,
    pub(crate) callback_var: Option<&'a HashMap<String, String>>,
}

/*
policy 示例：
{
    "expiration": "2023-12-03T13:00:00.000Z",
    "conditions": [
        // 这类一般用`对象写法`表达精确匹配
        {"bucket": "examplebucket"},
        {"x-oss-signature-version": "OSS4-HMAC-SHA256"},
        {"x-oss-credential": "AKIDEXAMPLE/20231203/cn-hangzhou/oss/aliyun_v4_request"},
        {"x-oss-security-token": "CAIS******"},
        {"x-oss-date": "20231203T121212Z"},
        // content-length-range 只能用：
        ["content-length-range", 1, 10],
        // 可以用 eq / starts-with / in / not-in 的字段
        ["eq", "$success_action_status", "201"],
        ["starts-with", "$key", "user/eric/"],
        ["in", "$content-type", ["image/jpg", "image/png"]],
        ["not-in", "$cache-control", ["no-cache"]]
    ]
}

前端拿到policy后请求的表单域示例：
{
    // 要求必带的字段
    policy: "xxx",  // 后端传过来的policy，
    key： "xxx"，  // 如果policy有指定的限制则按照限制填写，否则可以是任意值
    // ...其他policy中指定的PostObject API 表单元素和内容规则
    file: <file>,  // 最后一个字段必须是file
}
*/

// 把 PostPolicyCondition 转成 Vec<serde_json::Value>
fn conditions_to_json_array(cond: &PostPolicyCondition) -> Vec<Value> {
    let mut arr = Vec::new();

    // {"bucket": "examplebucket"}
    if let Some(bucket) = &cond.bucket {
        arr.push(json!({ "bucket": bucket }));
    }

    // {"x-oss-signature-version": "OSS4-HMAC-SHA256"}
    arr.push(json!({
        "x-oss-signature-version": &cond.x_oss_signature_version
    }));

    // {"x-oss-credential": "..."}
    arr.push(json!({
        "x-oss-credential": &cond.x_oss_credential
    }));

    // {"x-oss-security-token": "..."}
    if let Some(token) = &cond.x_oss_security_token {
        arr.push(json!({
            "x-oss-security-token": token
        }));
    }

    // {"x-oss-date": "..."}
    arr.push(json!({
        "x-oss-date": &cond.x_oss_date
    }));

    // ["content-length-range", 1, 10]
    if let Some((min, max)) = cond.content_length_range {
        arr.push(json!(["content-length-range", min, max]));
    }

    // ["eq", "$success_action_status", "201"]
    if let Some((op, value)) = &cond.success_action_status {
        // 文档字段名称就是下划线success_action_status
        arr.push(json!([op, "$success_action_status", value]));
    }

    // ["starts-with", "$key", "user/eric/"]
    if let Some((op, value)) = &cond.key {
        arr.push(json!([op, "$key", value]));
    }

    // ["in", "$content-type", ["image/jpg", "image/png"]]
    if let Some((op, values)) = &cond.content_type {
        arr.push(json!([op, "$content-type", values]));
    }

    // ["not-in", "$cache-control", ["no-cache"]]
    if let Some((op, values)) = &cond.cache_control {
        arr.push(json!([op, "$cache-control", values]));
    }

    // ==========================================================
    // PostObject API 表单元素（字段）
    // chatgpt 说`Cache-Control, Content-Type, Content-Disposition, Content-Encoding, Expires
    // 等 HTTP Header 作为表单域传递，支持精确匹配和前缀匹配方式。
    if let Some((op, value)) = &cond.content_disposition {
        arr.push(json!([op, "$content-disposition", value]));
    }
    if let Some(encoding) = &cond.content_encoding {
        arr.push(json!(["eq", "$content-encoding", encoding]));
    }
    if let Some(expires) = &cond.expires {
        arr.push(json!(["eq", "$expires", expires.format(&Rfc2822).unwrap()]));
    }
    if let Some(encryption) = &cond.x_oss_server_side_data_encryption {
        arr.push(json!([
            "eq",
            "$x-oss-server-side-data-encryption",
            encryption
        ]));
    }
    if let Some(key_id) = &cond.x_oss_server_side_encryption_key_id {
        arr.push(json!([
            "eq",
            "$x-oss-server-side-encryption-key-id",
            key_id
        ]));
    }
    if let Some(content_type) = &cond.x_oss_content_type {
        arr.push(json!(["eq", "$x-oss-content-type", content_type]));
    }
    if let Some(forbid) = &cond.x_oss_forbid_overwrite {
        arr.push(json!(["eq", "$x-oss-forbid-overwrite", forbid]));
    }
    if let Some(acl) = &cond.x_oss_object_acl {
        arr.push(json!(["eq", "$x-oss-object-acl", acl]));
    }
    if let Some(storage_class) = &cond.x_oss_storage_class {
        arr.push(json!(["eq", "$x-oss-storage-class", storage_class]));
    }
    if let Some((op, value)) = &cond.success_action_redirect {
        arr.push(json!([op, "$success_action_redirect", value]));
    }
    if !cond.custom_metas.is_empty() {
        for (key, (op, value)) in &cond.custom_metas {
            arr.push(json!([op, format!("${}", key), value]));
        }
    }

    // callback处理
    if let Some(s) = &cond.callback_b64 {
        arr.push(json!(["eq", "$callback", s]));
    }

    if let Some(vars) = cond.callback_var {
        for (var_key, var_value) in vars {
            arr.push(json!(["eq", format!("${}", var_key), var_value]));
        }
    }

    arr
}

fn serialize_conditions<S>(cond: &PostPolicyCondition, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let arr = conditions_to_json_array(cond);
    arr.serialize(serializer)
}

#[derive(Debug)]
pub struct GeneratePolicyResult {
    /// 固定为`OSS4-HMAC-SHA256`
    pub x_oss_signature_version: String,
    pub x_oss_credential: String,
    pub x_oss_date: String,
    pub x_oss_signature: String,
    /// base64编码后的policy字符串
    pub policy: String,
    /// base64编码后的callback字符串
    pub callback: Option<String>,
    pub callback_var: Option<HashMap<String, String>>,
    /// 如果使用了STS临时密钥，需要在请求头中添加`x-oss-security-token`
    pub x_oss_security_token: Option<String>,
}

// endregion

// region:    --- get object
/* 想做成嵌套的builder的，即类似于：
    GetObject::builder()
    .header_range("bytes=0-9")
    .header_if_modified_since("...")
    .query_response_content_language("en-US")
    .build();
    然后会自动收集到：
    GetObject {
        header: GetObjectHeaders { ... },
        query: GetObjectQueryParams { ... },
    }
    但bon不支持嵌套builder，所以只能把请求头和请求参数都放在同一个builder里，然后手动区分
*/

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct GetObjectHeaders<'a> {
    pub(crate) range: Option<&'a str>,
    pub(crate) if_modified_since: Option<&'a str>,
    pub(crate) if_unmodified_since: Option<&'a str>,
    pub(crate) if_match: Option<&'a str>,
    pub(crate) if_none_match: Option<&'a str>,
    pub(crate) accept_encoding: Option<&'a str>,
}
#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct GetObjectQueries<'a> {
    pub(crate) response_content_language: Option<&'a str>,
    pub(crate) response_expires: Option<&'a str>,
    pub(crate) response_cache_control: Option<&'a str>,
    pub(crate) response_content_disposition: Option<&'a str>,
    pub(crate) response_content_encoding: Option<&'a str>,
}
#[derive(Builder)]
pub struct GetObject<'a> {
    #[builder(start_fn)]
    pub(crate) client: &'a Client,

    // GetObject API的请求头
    pub(crate) range: Option<&'a str>,
    pub(crate) if_modified_since: Option<&'a str>,
    pub(crate) if_unmodified_since: Option<&'a str>,
    pub(crate) if_match: Option<&'a str>,
    pub(crate) if_none_match: Option<&'a str>,
    pub(crate) accept_encoding: Option<&'a str>,

    // GetObject API的请求参数
    pub(crate) response_content_language: Option<&'a str>,
    pub(crate) response_expires: Option<&'a str>,
    pub(crate) response_cache_control: Option<&'a str>,
    pub(crate) response_content_disposition: Option<&'a str>,
    pub(crate) response_content_encoding: Option<&'a str>,
}

impl GetObject<'_> {
    pub(crate) fn headers_part(&self) -> GetObjectHeaders<'_> {
        GetObjectHeaders {
            range: self.range,
            if_modified_since: self.if_modified_since,
            if_unmodified_since: self.if_unmodified_since,
            if_match: self.if_match,
            if_none_match: self.if_none_match,
            accept_encoding: self.accept_encoding,
        }
    }

    pub(crate) fn queries_part(&self) -> GetObjectQueries<'_> {
        GetObjectQueries {
            response_content_language: self.response_content_language,
            response_expires: self.response_expires,
            response_cache_control: self.response_cache_control,
            response_content_disposition: self.response_content_disposition,
            response_content_encoding: self.response_content_encoding,
        }
    }
}

#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GetObjectResponseHeader {
    #[serde(default, skip_deserializing)]
    pub custom_x_oss_meta: HashMap<String, String>,
    pub x_oss_server_side_encryption: Option<String>,
    pub x_oss_tagging_count: Option<String>,
    pub x_oss_expiration: Option<String>,
    #[serde_as(as = "DisplayFromStr")]
    pub content_length: u64,
    pub accept_ranges: Option<String>,
    pub content_type: String,
    pub last_modified: String,
    pub etag: String,
}
// endregion: --- get object

// region:    --- copy object
fn validate_source_name(source: &str) -> Result<(), Error> {
    if source.is_empty() {
        return Err(Error::Common(
            "x-oss-copy-source cannot be empty".to_owned(),
        ));
    }

    if !source.starts_with('/') {
        return Err(Error::Common(
            "x-oss-copy-source must start with '/'".to_owned(),
        ));
    }

    match validate_object_name(&source[1..]) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::Common(format!(
            "x-oss-copy-source is invalid: {}",
            e
        ))),
    }
}

#[serde_with::skip_serializing_none]
#[derive(Builder, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CopyObject<'a> {
    #[builder(start_fn)]
    #[serde(skip_serializing)]
    pub(crate) client: &'a Client,

    x_oss_forbid_overwrite: Option<&'a str>,
    #[builder(with = |s: &'a str| ->Result<_, Error> {
        validate_source_name(s)?;
        Ok(s)
    })]
    pub(crate) x_oss_copy_source: &'a str,
    x_oss_copy_source_if_match: Option<&'a str>,
    x_oss_copy_source_if_none_match: Option<&'a str>,
    x_oss_copy_source_if_unmodified_since: Option<&'a str>,
    x_oss_copy_source_if_modified_since: Option<&'a str>,
    x_oss_metadata_directive: Option<&'a str>,
    x_oss_server_side_encryption: Option<&'a str>,
    x_oss_server_side_encryption_key_id: Option<&'a str>,
    x_oss_object_acl: Option<&'a str>,
    x_oss_storage_class: Option<&'a str>,
    x_oss_tagging: Option<&'a str>,
    x_oss_tagging_directive: Option<&'a str>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CopyObjectResult {
    pub e_tag: String,
    pub last_modified: String,
}
// endregion: --- copy object

// region:    --- append object
#[serde_with::skip_serializing_none]
#[derive(Builder, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct AppendObject<'a> {
    #[serde(skip_serializing)]
    #[builder(start_fn)]
    pub(crate) client: &'a Client,

    // api请求头
    // x-oss-meta-* Header
    #[serde(skip_serializing)]
    #[builder(field)]
    pub(crate) custom_metas: HashMap<String, String>,
    // append, position添加到url的query中;append不添加到header中
    cache_control: Option<&'a str>,
    content_disposition: Option<&'a str>,
    // content_md5 自动添加
    expires: Option<&'a str>,
    x_oss_server_side_encryption: Option<&'a str>,
    x_oss_object_acl: Option<&'a str>,
    x_oss_storage_class: Option<&'a str>,
    x_oss_tagging: Option<&'a str>,

    // 公共请求头
    content_type: Option<&'a str>,
    // content_length  自动添加
}

impl<'a, S: append_object_builder::State> OssMetaExt<'a> for AppendObjectBuilder<'a, S> {
    fn custom_metas_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.custom_metas
    }
}

// endregion: --- append object

// region:    --- delete object
#[derive(Debug)]
pub struct DeleteObjectResponseHeader {
    pub x_oss_delete_marker: Option<bool>,
    pub x_oss_version_id: Option<String>,
}
// endregion

// region:    --- delete_multiple_objects
#[derive(Builder)]
pub struct DeleteMultipleObjects<'a> {
    #[builder(start_fn)]
    pub(crate) client: &'a Client,

    // api请求头
    pub(crate) encoding_type: Option<&'a str>,
    // Content-Length 自动添加
    // Content-MD5 自动添加

    // 请求元素
    pub(crate) objects: Vec<ObjectToDelete<'a>>,
    pub(crate) quiet: bool,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct DeleteMultipleObjectsRequest<'a> {
    pub quiet: bool,
    pub object: &'a Vec<ObjectToDelete<'a>>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ObjectToDelete<'a> {
    pub key: &'a str,
    pub version_id: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteResult {
    pub deleted: Vec<Deleted>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Deleted {
    pub key: String,
    pub delete_marker: Option<bool>,
    pub delete_marker_version_id: Option<String>,
    pub version_id: Option<String>,
}
// endregion: --- delete_multiple_objects

// region:    --- head object
#[serde_with::skip_serializing_none]
#[derive(Builder, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct HeadObject<'a> {
    #[builder(start_fn)]
    #[serde(skip_serializing)]
    pub(crate) client: &'a Client,

    pub if_modified_since: Option<&'a str>,
    pub if_unmodified_since: Option<&'a str>,
    pub if_match: Option<&'a str>,
    pub if_none_match: Option<&'a str>,
}

#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct HeadObjectResponseHeader {
    #[serde(default, skip_deserializing)]
    pub custom_x_oss_meta: HashMap<String, String>,
    pub x_oss_server_side_encryption: Option<String>,
    pub x_oss_server_side_encryption_key_id: Option<String>,
    pub x_oss_storage_class: String,
    pub x_oss_object_type: String,
    pub x_oss_next_append_position: Option<String>,
    pub x_oss_hash_crc64ecma: Option<String>,
    pub x_oss_transition_time: Option<String>,
    pub x_oss_expiration: Option<String>,
    pub x_oss_restore: Option<String>,
    pub x_oss_process_status: Option<String>,
    pub x_oss_request_charged: Option<String>,
    pub content_md5: Option<String>,
    pub last_modified: String,
    pub access_control_allow_origin: Option<String>,
    pub access_control_allow_methods: Option<String>,
    pub access_control_max_age: Option<String>,
    pub access_control_allow_headers: Option<String>,
    pub access_control_expose_headers: Option<String>,
    pub x_oss_tagging_count: Option<String>,
    pub content_type: String,
    #[serde_as(as = "DisplayFromStr")]
    pub content_length: u64,
    pub etag: String,
}
// endregion: --- head object

// region get object meta
#[serde_as]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct GetObjectMetaResponseHeader {
    #[serde_as(as = "DisplayFromStr")]
    pub content_length: u64,
    pub etag: String,
    pub x_oss_transition_time: Option<String>,
    pub x_oss_last_access_time: Option<String>,
    pub last_modified: String,
    pub x_oss_version_id: Option<String>,
}
// endregion

// region callback

/// [oss callback文档](https://help.aliyun.com/zh/oss/developer-reference/callback#19f2e6eb46b27)
#[derive(Builder)]
pub struct OssCallBack {
    #[builder(field)]
    // callback_url是必须要有的参数，但是这里使用Vec来收集(builder(field))，会使得这个参数变成非必须的了
    callback_url: Vec<String>,
    pub(crate) callback_body: CallBackBody,
    callback_host: Option<String>,
    callback_sni: Option<bool>,
    callback_body_type: Option<CallbackBodyType>,
}

#[derive(Serialize, Debug)]
pub enum CallbackBodyType {
    #[serde(rename = "application/json")]
    Json,
    #[serde(rename = "application/x-www-form-urlencoded")]
    UrlEncoded,
}

impl<S: oss_call_back_builder::State> OssCallBackBuilder<S> {
    pub fn callback_url<'a>(mut self, urls: impl IntoIterator<Item = &'a str>) -> Self {
        for url in urls {
            self.callback_url.push(url.to_string());
        }
        self
    }
}

fn callback_url_serialize<S: Serializer>(
    urls: &[String],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    // 定义一个编码集合：我们希望编码所有非字母／数字／-_.~，并且还要编码空格、中文、特殊符号等
    const ENCODE_SET: &AsciiSet = &CONTROLS
        .add(b' ') // 空格
        .add(b'"') // 引号
        .add(b'<')
        .add(b'>')
        .add(b'`');
    // 你也可以添加更多你想强制编码的字符
    let mut v = vec![];
    for url in urls {
        let encoded_url = utf8_percent_encode(url, ENCODE_SET).to_string();
        v.push(encoded_url);
    }
    let res = v.join(";");
    serializer.serialize_str(&res)
}

impl Serialize for OssCallBack {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let callback_body_serialized = match &self.callback_body_type {
            Some(CallbackBodyType::Json) => self.callback_body.to_serialized_json_string(),
            _ => self.callback_body.to_serialized_kv_string(),
        };

        #[serde_with::skip_serializing_none]
        #[derive(Serialize, Debug)]
        #[serde(rename_all = "camelCase")]
        struct OssCallBackHelper<'a> {
            // 这里继续复用你原来的自定义序列化函数
            #[serde(serialize_with = "callback_url_serialize")]
            callback_url: &'a Vec<String>,
            callback_body: String,
            callback_host: Option<&'a String>,
            #[serde(rename = "callbackSNI")]
            callback_sni: Option<bool>,
            callback_body_type: Option<&'a CallbackBodyType>,
        }

        let helper = OssCallBackHelper {
            callback_url: &self.callback_url,
            callback_body: callback_body_serialized,
            callback_host: self.callback_host.as_ref(),
            callback_sni: self.callback_sni,
            callback_body_type: self.callback_body_type.as_ref(),
        };

        helper.serialize(serializer)
    }
}

#[derive(Builder)]
pub struct CallBackBody {
    // callback-var
    #[builder(field)]
    // 表示形式: (callbackBody中的key, callback-var中的key, value)，最终会组成:
    // callbackBody: "key=${x:var_key}", callback-var: {"x:var_key": "value"}
    pub(crate) callback_var: Vec<(String, String, String)>,
    // callbackBody支持的系统参数
    #[builder(default)]
    bucket: bool,
    #[builder(default)]
    object: bool,
    #[builder(default)]
    e_tag: bool,
    #[builder(default)]
    size: bool,
    #[builder(default)]
    mime_type: bool,
    #[builder(default)]
    image_info_height: bool,
    #[builder(default)]
    image_info_width: bool,
    #[builder(default)]
    image_info_format: bool,
    #[builder(default)]
    crc64: bool,
    #[builder(default)]
    content_md5: bool,
    #[builder(default)]
    vpc_id: bool,
    #[builder(default)]
    client_ip: bool,
    #[builder(default)]
    req_id: bool,
    #[builder(default)]
    operation: bool,
}

impl<S: call_back_body_builder::State> CallBackBodyBuilder<S> {
    fn var_key_string(var_key: &str) -> String {
        format!("x:{}", var_key)
    }
    /// 设置callbackBody中的key，callback-var中的key和callback-var中的value
    pub fn var(mut self, body_key: &str, var_key: &str, var_value: &str) -> Self {
        self.callback_var.push((
            body_key.to_owned(),
            Self::var_key_string(var_key),
            var_value.to_owned(),
        ));
        self
    }

    /// (body_key, var_key, var_value) 三元组的批量添加
    pub fn vars<'a>(mut self, vars: impl IntoIterator<Item = (&'a str, &'a str, &'a str)>) -> Self {
        for (body_key, var_key, var_value) in vars {
            self.callback_var.push((
                body_key.to_owned(),
                Self::var_key_string(var_key),
                var_value.to_owned(),
            ));
        }
        self
    }
}

impl CallBackBody {
    fn visit_fields<F>(&self, mut f: F)
    where
        // key 用 &str，value 用 String 方便 callback_var 里动态构造
        F: FnMut(&str, String),
    {
        if self.bucket {
            f("bucket", "${bucket}".to_owned());
        }
        if self.object {
            f("object", "${object}".to_owned());
        }
        if self.e_tag {
            f("etag", "${etag}".to_owned());
        }
        if self.size {
            f("size", "${size}".to_owned());
        }
        if self.mime_type {
            f("mimeType", "${mimeType}".to_owned());
        }
        if self.image_info_height {
            f("imageInfo.height", "${imageInfo.height}".to_owned());
        }
        if self.image_info_width {
            f("imageInfo.width", "${imageInfo.width}".to_owned());
        }
        if self.image_info_format {
            f("imageInfo.format", "${imageInfo.format}".to_owned());
        }
        if self.crc64 {
            f("crc64", "${crc64}".to_owned());
        }
        if self.content_md5 {
            f("contentMd5", "${contentMd5}".to_owned());
        }
        if self.vpc_id {
            f("vpcId", "${vpcId}".to_owned());
        }
        if self.client_ip {
            f("clientIp", "${clientIp}".to_owned());
        }
        if self.req_id {
            f("reqId", "${reqId}".to_owned());
        }
        if self.operation {
            f("operation", "${operation}".to_owned());
        }

        for (k1, k2, _) in &self.callback_var {
            // k1: 自定义 key，k2: 自定义变量名
            f(k1, format!("${{{}}}", k2));
        }
    }

    /// 生成 KV 形式：bucket=${bucket}&object=${object}&...
    pub(crate) fn to_serialized_kv_string(&self) -> String {
        let mut body_list = Vec::new();

        self.visit_fields(|k, v| {
            body_list.push(format!("{k}={v}"));
        });

        body_list.join("&")
    }

    /// 生成类似于JSON形式：{"bucket": ${bucket}, ...};注意：这里的value部分没有加引号，是占位符形式，例如：${bucket}
    pub(crate) fn to_serialized_json_string(&self) -> String {
        let mut s = String::new();
        let mut first = true;
        s.push('{');

        self.visit_fields(|k, v| {
            if !first {
                s.push(',');
            } else {
                first = false;
            }

            // 文档里没有说构建时处理转义json的问题，所以这里直接拼接字符串

            // 写 key 部分：`"bucket":`
            s.push('"');
            s.push_str(k);
            s.push('"');
            s.push(':');

            // 写 value 部分，直接拼占位符，例如 ${bucket}
            s.push_str(&v);
        });

        s.push('}');

        // println!("callback body string: {}", s);

        s
    }
}
// endregion

#[derive(Debug)]
pub struct PresignedUrlResult {
    pub url: String,
    /// 如果使用了STS临时密钥签名，请求的时候请求头需要添加`x-oss-security-token`
    pub x_oss_security_token: Option<String>,
}
