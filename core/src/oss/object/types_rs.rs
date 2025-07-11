use crate::oss::utils::SerializeToHashMap;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::collections::HashMap;

// region:    --- pub object
/// Header字段中：
/// - content_md5: 由程序自动添加
/// - content_length：由程序自动添加
/// - e_tag：不添加
#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct PutObjectHeader<'a> {
    // 公共请求头
    /// 对于MIME不会进行检查合法性检查
    pub content_type: Option<&'a str>,
    // content_length  自动添加

    // api请求头
    pub cache_control: Option<&'a str>,
    pub content_disposition: Option<&'a str>,
    pub content_encoding: Option<&'a str>,
    // content_md5  自动添加
    // e_tag  不添加
    pub expires: Option<&'a str>,
    pub x_oss_forbid_overwrite: Option<&'a str>,
    pub x_oss_server_side_encryption: Option<&'a str>,
    pub x_oss_server_side_data_encryption: Option<&'a str>,
    pub x_oss_server_side_encryption_key_id: Option<&'a str>,
    pub x_oss_object_acl: Option<&'a str>,
    pub x_oss_storage_class: Option<&'a str>,
    pub x_oss_tagging: Option<&'a str>,
}

impl SerializeToHashMap for PutObjectHeader<'_> {}

/// x-oss-meta-* Header<br/>
/// 对于`XOtherHeader`中的key: value，会自动转换为: `x-oss-meta-key: value`，并添加到请求的Header
pub struct XMetaHeader<'a>(pub HashMap<&'a str, &'a str>);

impl XMetaHeader<'_> {
    pub fn get_meta_map(&self) -> HashMap<String, String> {
        self.0
            .iter()
            .map(|(k, v)| (format!("x-oss-meta-{k}"), v.to_string()))
            .collect()
    }
}
// endregion: --- pub object

// region:    --- get object
#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct GetObjectHeader<'a> {
    pub response_content_type: Option<&'a str>,
    pub response_content_language: Option<&'a str>,
    pub response_expires: Option<&'a str>,
    pub response_cache_control: Option<&'a str>,
    pub response_content_disposition: Option<&'a str>,
    pub response_content_encoding: Option<&'a str>,
    pub range: Option<&'a str>,
    pub if_modified_since: Option<&'a str>,
    pub if_unmodified_since: Option<&'a str>,
    pub if_match: Option<&'a str>,
    pub if_none_match: Option<&'a str>,
    pub accept_encoding: Option<&'a str>,
}

impl SerializeToHashMap for GetObjectHeader<'_> {}
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
