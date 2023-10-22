//! 关于Object操作/基础操作

use super::types::*;
use super::utils::get_local_file;
use crate::error::Error;
use crate::oss::object::utils::get_dest_path;
use crate::oss::utils::get_content_md5;
use crate::oss::utils::{into_header_map, now_gmt, sign_authorization};
use crate::oss::OSSClient;

use reqwest::StatusCode;
use std::collections::{BTreeMap, HashMap};

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
        x_meta_header: Option<XMetaHeader>,
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

        let mut oss_header_map = BTreeMap::new();
        oss_header_map.append(&mut x_header_map);
        if let Some(m) = x_meta_header {
            oss_header_map.append(&mut m.get_btree_map());
        }

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

        let header_map = into_header_map(header_map);

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

        let header_map = into_header_map(header_map);

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

    /// - `x_oss_copy_source`为请求头必带参数，这里抽离出来作为函数参数输入，eg: `/source_bucket_name/source_object_name`
    /// - `dest_bucket`如果为`None`，则为`OSSClinet`配置的`bucket`，eg：`oss-example`
    /// - `dest_end_point`，同上，eg：`oss-cn-hangzhou.aliyuncs.com`
    /// - `dest_oss_path`：你复制object到`dest_bucket`的路径，eg：`/dir1/abc.txt`
    pub async fn copy_object(
        &self,
        x_oss_copy_source: &str,
        dest_bucket: Option<&str>,
        dest_end_point: Option<&str>,
        dest_oss_path: &str,
        copy_object_x_header: CopyObjectXHeader<'_>,
    ) -> Result<CopyObjectResult, Error> {
        let mut x_header_map: BTreeMap<String, String> =
            serde_json::from_value(serde_json::to_value(copy_object_x_header).unwrap()).unwrap();
        x_header_map.insert("x-oss-copy-source".to_owned(), x_oss_copy_source.to_owned());

        let bucket = if let Some(s) = dest_bucket {
            s.to_owned()
        } else {
            self.bucket.to_owned()
        };
        let end_point = if let Some(p) = dest_end_point {
            p.to_owned()
        } else {
            self.endpoint.to_owned()
        };
        let host = format!("{}.{}", bucket, end_point);
        let now_gmt = now_gmt();
        let authorization = sign_authorization(
            &self.access_key_id,
            &self.access_key_secret,
            "PUT",
            None,
            None,
            &now_gmt,
            Some(&x_header_map),
            Some(&bucket),
            Some(&dest_oss_path[1..]),
        );
        let mut common_header = self.get_common_header_map(&authorization, None, None, &now_gmt);
        common_header.insert("Host".to_owned(), host);
        common_header.extend(x_header_map);

        let header_map = into_header_map(common_header);

        let builder = self
            .http_client
            .put(format!("https://{}.{}{}", bucket, end_point, dest_oss_path))
            .headers(header_map);
        // println!("builder: {:#?}", builder);
        let resp = builder.send().await?;
        if resp.status() != StatusCode::OK {
            return Err(Error::StatusCodeNot200Resp(resp));
        }

        let text = resp.text().await?;
        let res = quick_xml::de::from_str(&text).map_err(|e| Error::XMLDeError {
            source: e,
            origin_text: text,
        })?;

        Ok(res)
    }

    /// - 当创建一个新的Appendable Object的时候，`position`设为`0`，如果该object已存在，则`position`为该Object的字节大小，即此次append object的起始位置
    #[allow(clippy::too_many_arguments)]
    pub async fn append_object(
        &self,
        bytes: Vec<u8>,
        dest_path: &str,
        content_type: Option<&str>,
        position: u64,
        c_header: AppendObjectCHeader<'_>,
        x_header: AppendObjectXHeader<'_>,
        x_meta_header: Option<XMetaHeader>,
    ) -> Result<AppendObjectResponseHeaderInfo, Error> {
        let mut header_map = HashMap::new();
        let c_header_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(c_header).unwrap()).unwrap();
        header_map.extend(c_header_map);

        let mut x_header_map: BTreeMap<String, String> =
            serde_json::from_value(serde_json::to_value(x_header).unwrap()).unwrap();
        if let Some(m) = x_meta_header {
            x_header_map.append(&mut m.get_btree_map());
        }

        let content_type = if let Some(s) = content_type {
            s.to_owned()
        } else {
            mime_guess::MimeGuess::from_path(dest_path)
                .first_or_octet_stream()
                .to_string()
        };
        let now_gmt = now_gmt();
        let contet_md5 = get_content_md5(&bytes);
        // `https://bucket.endpoint`之后的内容为object_path，`&object_path[1..]`即为此次的`object_name`
        let object_path = format!("{}?append&position={}", dest_path, position);
        let authorization = sign_authorization(
            &self.access_key_id,
            &self.access_key_secret,
            "POST",
            Some(&contet_md5),
            Some(&content_type),
            &now_gmt,
            Some(&x_header_map),
            Some(&self.bucket),
            Some(&object_path[1..]),
        );

        header_map.extend(x_header_map);
        let common_header_map = self.get_common_header_map(
            &authorization,
            Some(&bytes.len().to_string()),
            Some(&content_type),
            &now_gmt,
        );
        header_map.extend(common_header_map);
        header_map.insert("Content-MD5".to_owned(), contet_md5);
        header_map.insert("Position".to_owned(), position.to_string());

        let header_map = into_header_map(header_map);

        let builder = self
            .http_client
            .post(format!("{}{}", self.bucket_url(), object_path))
            .headers(header_map)
            .body(bytes);
        // println!("builder: {:#?}", builder);
        let resp = builder.send().await?;
        if resp.status() != StatusCode::OK {
            return Err(Error::StatusCodeNot200Resp(resp));
        }
        // println!("resp:{:#?}", resp);
        let resp_headers = resp.headers();
        // 下面直接使用unwrap()，默认不会出错，如果实际中出现错误，这里代码要优化
        let next_pos: u64 = resp_headers
            .get("x-oss-next-append-position")
            .unwrap()
            .to_str()
            .unwrap()
            .parse()
            .unwrap();
        let crc: u64 = resp_headers
            .get("x-oss-hash-crc64ecma")
            .unwrap()
            .to_str()
            .unwrap()
            .parse()
            .unwrap();

        Ok(AppendObjectResponseHeaderInfo {
            x_oss_next_append_position: next_pos,
            x_oss_hash_crc64ecma: crc,
        })
    }

    pub async fn delete_object(&self, oss_path: &str) -> Result<(), Error> {
        let now_gmt = now_gmt();
        let authorization = sign_authorization(
            &self.access_key_id,
            &self.access_key_secret,
            "DELETE",
            None,
            None,
            &now_gmt,
            None,
            Some(&self.bucket),
            Some(&oss_path[1..]),
        );

        let common_header = self.get_common_header_map(&authorization, None, None, &now_gmt);
        let header_map = into_header_map(common_header);

        let builder = self
            .http_client
            .delete(format!("{}{}", self.bucket_url(), oss_path))
            .headers(header_map);
        let resp = builder.send().await?;
        if resp.status() != StatusCode::NON_AUTHORITATIVE_INFORMATION {
            return Err(Error::StatusCodeNot200Resp(resp));
        }

        Ok(())
    }

    /// 注意，删除多个文件的时候，你的`DeleteObject::key`填的是文件名称，这和`put_object`的参数有所不同：
    /// 这里的key需要去掉`oss_path`前面的斜杠`\`，如：
    /// - oss_path: `/aa/123.txt`
    /// - key需要写为：`aa/123/txt`
    // TODO MAYBE：其它的所有代码，其实object_name应该为a/abc/c这种，然后代码自己添加/->/a/abc/c才合理。还是说这里统一一下使用/a/abc/c？
    pub async fn delete_multiple_objects(
        &self,
        encoding_type: Option<&str>,
        delete_objects: Vec<DeleteObject<'_>>,
        quiet_resp: bool,
    ) -> Result<Option<DeleteResult>, Error> {
        let delete_req = DeleteMultipleObjectsRequest {
            quiet: &quiet_resp.to_string(),
            object: delete_objects,
        };
        let req_body = quick_xml::se::to_string_with_root("Delete", &delete_req).unwrap();

        let content_length = req_body.len();
        let content_md5 = get_content_md5(req_body.as_bytes());
        let now_gmt = now_gmt();
        let authorization = sign_authorization(
            &self.access_key_id,
            &self.access_key_secret,
            "POST",
            Some(&content_md5),
            None,
            &now_gmt,
            None,
            Some(&self.bucket),
            Some("?delete"),
        );

        let mut common_header = self.get_common_header_map(
            &authorization,
            Some(&content_length.to_string()),
            None,
            &now_gmt,
        );
        common_header.insert("Content-MD5".to_owned(), content_md5);
        if let Some(s) = encoding_type {
            common_header.insert("Encoding-type".to_owned(), s.to_owned());
        }
        let header_map = into_header_map(common_header);

        let builder = self
            .http_client
            .post(format!("{}/?delete", self.bucket_url()))
            .headers(header_map)
            .body(req_body);
        let resp = builder.send().await?;
        if resp.status() != StatusCode::OK {
            return Err(Error::StatusCodeNot200Resp(resp));
        }

        if quiet_resp {
            return Ok(None);
        }

        let text = resp.text().await?;
        let res = quick_xml::de::from_str(&text).map_err(|e| Error::XMLDeError {
            source: e,
            origin_text: text,
        })?;

        Ok(Some(res))
    }

    pub async fn head_object(
        &self,
        oss_path: &str,
        req_header: HeadObjectHeader<'_>,
    ) -> Result<HashMap<String, String>, Error> {
        let now_gmt = now_gmt();
        let authorization = sign_authorization(
            &self.access_key_id,
            &self.access_key_secret,
            "HEAD",
            None,
            None,
            &now_gmt,
            None,
            Some(&self.bucket),
            Some(&oss_path[1..]),
        );

        let mut common_header = self.get_common_header_map(&authorization, None, None, &now_gmt);
        let req_header_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(&req_header).unwrap()).unwrap();
        common_header.extend(req_header_map);
        let header_map = into_header_map(common_header);

        let builder = self
            .http_client
            .head(format!("{}{}", self.bucket_url(), oss_path))
            .headers(header_map);
        let resp = builder.send().await?;
        if resp.status() != StatusCode::OK {
            return Err(Error::StatusCodeNot200Resp(resp));
        }
        let response_header: HashMap<String, String> = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_owned()))
            .collect();

        Ok(response_header)
    }

    /// - 这里返回`HashMap`而没有返回struct，主要考虑到response header中有一些参数文档中没说出来，不便于转化为指定的struct
    /// - 返回的`HashMap`中所有的`key`均为小写，这里代码并没有使用`to_lowercase`，因为`reqwest`获取的header都为小写
    pub async fn get_object_meta(&self, oss_path: &str) -> Result<HashMap<String, String>, Error> {
        let now_gmt = now_gmt();
        let authorization = sign_authorization(
            &self.access_key_id,
            &self.access_key_secret,
            "HEAD",
            None,
            None,
            &now_gmt,
            None,
            Some(&self.bucket),
            Some(&format!("{}?objectMeta", &oss_path[1..])),
        );

        let common_header = self.get_common_header_map(&authorization, None, None, &now_gmt);
        let header_map = into_header_map(common_header);

        let builder = self
            .http_client
            .head(format!("{}{}?objectMeta", self.bucket_url(), oss_path))
            .headers(header_map);
        let resp = builder.send().await?;
        if resp.status() != StatusCode::OK {
            return Err(Error::StatusCodeNot200Resp(resp));
        }
        let response_header: HashMap<String, String> = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_owned()))
            .collect();

        Ok(response_header)
    }
}
