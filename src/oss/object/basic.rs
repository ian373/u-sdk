//! 关于Object操作/基础操作

use super::utils::get_local_file;
use crate::error::Error;
use crate::oss::object::utils::get_dest_path;
use crate::oss::utils::{now_gmt, sign_authorization};
use crate::oss::OSSClient;

use crate::oss::utils::get_content_md5;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::StatusCode;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

// region:    --- pub object
/// 一般性Header</br>
/// 以下两个header由程序读取文件的时候获取相应信息并自动添加：<br/>
/// - `content_md5`
/// - `content_length`
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CHeader<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_disposition: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_encoding: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e_tag: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<&'a str>,
}

/// x-oss-xxx Header
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct XHeader<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_oss_forbid_overwrite: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_oss_server_side_encryption: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_oss_server_side_data_encryption: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_oss_server_side_encryption_key_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_oss_object_acl: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_oss_storage_class: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_oss_tagging: Option<&'a str>,
}

/// x-oss-meta-* Header<br/>
/// 对于`XOtherHeader`中的key: value，会自动转换为: `x-oss-meta-key: value`，并添加到请求的Header
pub type XOtherHeader<'a> = HashMap<&'a str, &'a str>;
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

impl OSSClient {
    /// 本API将会一次性读取文件到内存然后进行上传，请注意上传文件的大小以免爆内存<br/>
    /// - `content_type`，如果为None，则根据文件后缀名自动推测对应类型，但是不能保证推测100%正确
    /// - `dest_path`：使用linux文件风格(`/xx/xx`)，且必须使用绝对路径，即以`/`开头,
    /// 如果以`/`结尾，则使用上传文件的文件名称，如果以`/xxx.xx`结尾，则文件名使用`xxx.xx`<br/>
    /// - 注意，本代码无法解析包含`/.`和`/..`的路径，如果出现上述情况，会导致`object_name`无法正确得出，从而导致签名计算错误。后期可能会解决此类问题
    pub async fn put_object(
        &self,
        c_header: CHeader<'_>,
        x_header: XHeader<'_>,
        x_other_header: XOtherHeader<'_>,
        local_file_path: &str,
        dest_path: &str,
        content_type: Option<&str>,
    ) -> Result<(), Error> {
        let (local_file_name, bytes) = get_local_file(local_file_path)?;

        let content_type = if let Some(s) = content_type {
            s.to_owned()
        } else {
            mime_guess::MimeGuess::from_path(&local_file_name)
                .first_or_octet_stream()
                .to_string()
        };

        let mut header_map = HashMap::new();
        let c_header_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(c_header).unwrap()).unwrap();
        header_map.extend(c_header_map);

        header_map.insert("Content-Length".to_owned(), bytes.len().to_string());

        let mut x_header_map: BTreeMap<String, String> =
            serde_json::from_value(serde_json::to_value(x_header).unwrap()).unwrap();
        let x_other_header_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(x_other_header).unwrap()).unwrap();
        let mut x_other_header_map = x_other_header_map
            .into_iter()
            .map(|(k, v)| (format!("x-oss-meta-{k}"), v))
            .collect();

        let mut oss_header_map = BTreeMap::new();
        oss_header_map.append(&mut x_header_map);
        oss_header_map.append(&mut x_other_header_map);

        let now_gmt = now_gmt();
        let dest_path = get_dest_path(dest_path, &local_file_name)?;
        let content_md5 = get_content_md5(&bytes);
        let authorization = sign_authorization(
            &self.access_key_id,
            &self.access_key_secret,
            "PUT",
            Some(&content_md5),
            Some(&content_type),
            &now_gmt,
            Some(&oss_header_map),
            Some(&self.bucket),
            // object_name不包含dest_path的第一个字符'/'
            Some(&dest_path[1..]),
        );
        header_map.insert("Content-MD5".to_owned(), content_md5);

        header_map.extend(oss_header_map);

        let common_header = self.get_common_header_map(
            &authorization,
            Some(&bytes.len().to_string()),
            Some(&content_type),
            &now_gmt,
        );
        header_map.extend(common_header);

        let header_map: HeaderMap = header_map
            .iter()
            .map(|(k, v)| {
                let name = HeaderName::from_bytes(k.as_bytes()).unwrap();
                let value = HeaderValue::from_bytes(v.as_bytes()).unwrap();
                (name, value)
            })
            .collect();

        let builder = self
            .http_client
            .put(format!("{}{}", self.bucket_url(), dest_path))
            .headers(header_map)
            .body(bytes);
        // println!("builder: {:#?}", builder);
        let resp = builder.send().await?;
        if resp.status() != StatusCode::OK {
            return Err(Error::StatusCodeNot200Resp(resp));
        }

        Ok(())
    }

    /// 获取单个文件或其部分数据，一次性读取数据到内存，然后再保存到本地磁盘中<br/>
    /// - `oss_path`：oss上object的绝对路径，linux路径风格，不能包含`/.`或`/..`
    /// - `dest_path`: 保存在本地磁盘的路径，需要一个文件的绝对路径
    /// - RETURN：当`c_header`的值全为`None`是，说明是一个普通文件下载，成功请求后函数不返回任何内容；
    /// 当`c_header`的值不全为`None`，说明对返回结果有一些配置，需要用到Response Header，这里把响应头转化为`HashMap`，用户可从中取得相应的内容
    pub async fn get_object(
        &self,
        c_header: GetObjectHeader<'_>,
        oss_path: &str,
        dest_path: &str,
    ) -> Result<Option<HashMap<String, String>>, Error> {
        // TODO 加强对路径合法性的校验，防止因为路径不合法导致的错误

        let now_gmt = now_gmt();
        let authorization = sign_authorization(
            &self.access_key_id,
            &self.access_key_secret,
            "GET",
            None,
            None,
            &now_gmt,
            None,
            Some(&self.bucket),
            Some(&oss_path[1..]),
        );

        let c_header_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(c_header).unwrap()).unwrap();
        let c_header_is_empty = c_header_map.is_empty();

        let common_header = self.get_common_header_map(&authorization, None, None, &now_gmt);
        let mut header_map = HashMap::new();
        header_map.extend(common_header);
        header_map.extend(c_header_map);

        let header_map: HeaderMap = header_map
            .iter()
            .map(|(k, v)| {
                let name = HeaderName::from_bytes(k.as_bytes()).unwrap();
                let value = HeaderValue::from_bytes(v.as_bytes()).unwrap();
                (name, value)
            })
            .collect();

        let builder = self
            .http_client
            .get(format!("{}{}", self.bucket_url(), oss_path))
            .headers(header_map);
        let resp = builder.send().await?;
        // println!("resp:{:#?}", resp);
        if resp.status() != StatusCode::OK && resp.status() != StatusCode::PARTIAL_CONTENT {
            return Err(Error::StatusCodeNot200Resp(resp));
        }

        if c_header_is_empty {
            std::fs::write(dest_path, resp.bytes().await?)
                .map_err(|e| Error::CommonError(format!("write data to disk error, {}", e)))?;
            Ok(None)
        } else {
            let map: HashMap<String, String> = resp
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_owned()))
                .collect();
            std::fs::write(dest_path, resp.bytes().await?)
                .map_err(|e| Error::CommonError(format!("write data to disk error, {}", e)))?;
            Ok(Some(map))
        }
    }
}
