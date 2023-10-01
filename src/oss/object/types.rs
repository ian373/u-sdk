use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

// region:    --- pub object
/// 一般性Header</br>
/// 以下两个header由程序读取文件的时候获取相应信息并自动添加：<br/>
/// - `content_md5`
/// - `content_length`
#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct CHeader<'a> {
    pub cache_control: Option<&'a str>,
    pub content_disposition: Option<&'a str>,
    pub content_encoding: Option<&'a str>,
    pub e_tag: Option<&'a str>,
    pub expires: Option<&'a str>,
}

/// x-oss-xxx Header
#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct XHeader<'a> {
    pub x_oss_forbid_overwrite: Option<&'a str>,
    pub x_oss_server_side_encryption: Option<&'a str>,
    pub x_oss_server_side_data_encryption: Option<&'a str>,
    pub x_oss_server_side_encryption_key_id: Option<&'a str>,
    pub x_oss_object_acl: Option<&'a str>,
    pub x_oss_storage_class: Option<&'a str>,
    pub x_oss_tagging: Option<&'a str>,
}

/// x-oss-meta-* Header<br/>
/// 对于`XOtherHeader`中的key: value，会自动转换为: `x-oss-meta-key: value`，并添加到请求的Header
pub struct XMetaHeader(BTreeMap<String, String>);

impl XMetaHeader {
    pub fn get_btree_map(self) -> BTreeMap<String, String> {
        self.0
    }
}

impl From<HashMap<&str, &str>> for XMetaHeader {
    fn from(value: HashMap<&str, &str>) -> Self {
        let map = value
            .into_iter()
            .map(|(k, v)| (format!("x-oss-meta-{k}"), v.to_owned()))
            .collect();
        Self(map)
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
// endregion: --- get object

// region:    --- copy object
#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct CopyObjectXHeader<'a> {
    pub x_oss_forbid_overwrite: Option<&'a str>,
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CopyObjectResult {
    pub e_tag: String,
    pub last_modified: String,
}
// endregion: --- copy object

// region:    --- append object
#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct AppendObjectCHeader<'a> {
    // contet_md5, position将根据函数自动添加
    pub cache_control: Option<&'a str>,
    pub content_disposition: Option<&'a str>,
    pub content_encoding: Option<&'a str>,
    pub expires: Option<&'a str>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct AppendObjectXHeader<'a> {
    pub x_oss_server_side_encryption: Option<&'a str>,
    pub x_oss_object_acl: Option<&'a str>,
    pub x_oss_storage_class: Option<&'a str>,
    pub x_oss_tagging: Option<&'a str>,
}

#[derive(Debug)]
pub struct AppendObjectResponseHeaderInfo {
    pub x_oss_next_append_position: u64,
    pub x_oss_hash_crc64ecma: u64,
}
// endregion: --- append object
