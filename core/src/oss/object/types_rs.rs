use super::super::Client;
use crate::oss::utils::SerializeToHashMap;
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

// x-oss-meta-* Header<br/>
// 对于`XOtherHeader`中的key: value，会自动转换为: `x-oss-meta-key: value`，并添加到请求的Header
impl<'a, S: put_object_builder::State> PutObjectBuilder<'a, S> {
    pub fn x_meta(mut self, key: &'a str, val: &'a str) -> Self {
        self.custom_metas
            .insert(format!("x-oss-meta-{key}"), val.to_owned());
        self
    }

    pub fn x_metas(mut self, metas: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        for (key, val) in metas {
            self.custom_metas
                .insert(format!("x-oss-meta-{key}"), val.to_owned());
        }
        self
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
// endregion: --- get object

// region:    --- copy object
#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct CopyObjectXHeader<'a> {
    pub x_oss_forbid_overwrite: Option<&'a str>,
    pub x_oss_copy_source: &'a str,
    pub x_oss_copy_source_if_match: Option<&'a str>,
    pub x_oss_copy_source_if_none_match: Option<&'a str>,
    pub x_oss_copy_source_if_unmodified_since: Option<&'a str>,
    pub x_oss_copy_source_if_modified_since: Option<&'a str>,
    pub x_oss_metadata_directive: Option<&'a str>,
    pub x_oss_server_side_encryption: Option<&'a str>,
    pub x_oss_server_side_encryption_key_id: Option<&'a str>,
    pub x_oss_object_acl: Option<&'a str>,
    pub x_oss_storage_class: Option<&'a str>,
    pub x_oss_tagging: Option<&'a str>,
    pub x_oss_tagging_directive: Option<&'a str>,
}

impl SerializeToHashMap for CopyObjectXHeader<'_> {}

pub struct CopyObjectDestInfo<'a> {
    pub region: &'a str,
    pub endpoint: &'a str,
    pub bucket: &'a str,
}
// endregion: --- copy object

// region:    --- append object
#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct AppendObjectHeader<'a> {
    // 公共请求头
    /// 对于MIME不会进行检查合法性检查
    pub content_type: Option<&'a str>,
    // content_length  自动添加

    // api请求头
    // append, position添加到url的query中;append不添加到header中
    #[serde_as(as = "DisplayFromStr")]
    pub position: u64,
    pub cache_control: Option<&'a str>,
    pub content_disposition: Option<&'a str>,
    pub content_encoding: Option<&'a str>,
    // content_md5自动添加
    pub expires: Option<&'a str>,
    pub x_oss_server_side_encryption: Option<&'a str>,
    pub x_oss_object_acl: Option<&'a str>,
    pub x_oss_storage_class: Option<&'a str>,
    pub x_oss_tagging: Option<&'a str>,
}

impl SerializeToHashMap for AppendObjectHeader<'_> {}
// endregion: --- append object

// region:    --- delete_multiple_objects
#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct DeleteMultipleObjectsRequest<'a> {
    pub quiet: bool,
    pub object: Vec<DeleteObject<'a>>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteObject<'a> {
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
#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct HeadObjectHeader<'a> {
    pub if_modified_since: Option<&'a str>,
    pub if_unmodified_since: Option<&'a str>,
    pub if_match: Option<&'a str>,
    pub if_none_match: Option<&'a str>,
}

impl SerializeToHashMap for HeadObjectHeader<'_> {}
// endregion: --- head object
