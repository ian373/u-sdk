use super::super::Client;
use crate::oss::Error;
use crate::oss::utils::validate_object_name;
use bon::Builder;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::collections::HashMap;
use std::path::Path;

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
    // x-meta-* Header，不序列化，需要的时候取出来自己整理
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

// region:    --- get object
#[serde_with::skip_serializing_none]
#[derive(Builder, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct GetObject<'a> {
    #[builder(start_fn)]
    #[serde(skip_serializing)]
    pub(crate) client: &'a Client,

    // 请求头
    range: Option<&'a str>,
    if_modified_since: Option<&'a str>,
    if_unmodified_since: Option<&'a str>,
    if_match: Option<&'a str>,
    if_none_match: Option<&'a str>,
    accept_encoding: Option<&'a str>,

    // 请求参数
    response_content_language: Option<&'a str>,
    response_expires: Option<&'a str>,
    response_cache_control: Option<&'a str>,
    response_content_disposition: Option<&'a str>,
    response_content_encoding: Option<&'a str>,
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
