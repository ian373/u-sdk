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

impl OSSClient {
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

    /// 创建一个文件夹，但其本质上上传一个以`/`结尾，0 bytes的object，用于模拟文件夹
    pub async fn create_virtual_dir() {
        todo!()
    }
}
