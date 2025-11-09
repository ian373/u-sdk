//! 关于Object操作/基础操作
//!
//! [官方文档](https://help.aliyun.com/zh/oss/developer-reference/basic-operations-1/)

use super::types_rs::*;
use crate::oss::Client;
use crate::oss::Error;
use crate::oss::sign_v4::HTTPVerb;
use crate::oss::utils::{
    compute_md5_from_file, get_content_md5, get_request_header, into_request_failed_error,
    parse_get_object_response_header, parse_xml_response, validate_object_name,
};
use bytes::Bytes;
use reqwest::Body;
use reqwest::header::HeaderMap;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio_stream::{Stream, StreamExt};
use tokio_util::io::ReaderStream;

impl<'a> PutObject<'a> {
    /// - `content_type`，不会进行MIME合法性检查
    /// - `object_name`：遵守OSS的Object[命名规则](https://help.aliyun.com/zh/oss/user-guide/object-naming-conventions)
    /// - `data`：如果需要创建文件夹，object_name以`/`结尾，`Vec`大小为0即可
    pub async fn send(
        &self,
        object_name: &'a str,
        object: PutObjectBody<'a>,
    ) -> Result<PutObjectResponseHeader, Error> {
        validate_object_name(object_name)?;

        let client = self.client;
        let request_url = url::Url::parse(&format!(
            "https://{}.{}/{}", // url不能添加`/`结尾，因为是否有`/`由object_name决定
            client.bucket, client.endpoint, object_name
        ))
        .unwrap();

        let mut req_header_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap();
        // 添加api剩下的请求头
        match &object {
            PutObjectBody::Bytes(bytes) => {
                req_header_map.insert("content-md5".to_owned(), get_content_md5(bytes.as_slice()));
                req_header_map.insert("content-length".to_owned(), bytes.len().to_string());
            }
            PutObjectBody::FilePath(path) => {
                let file_size = std::fs::metadata(path)?.len();
                req_header_map.insert("content-length".to_owned(), file_size.to_string());
                let md5_str = compute_md5_from_file(path).await?;
                req_header_map.insert("content-md5".to_owned(), md5_str);
            }
        }

        // 如果有x-meta-*，将其添加到请求头中
        if !self.custom_metas.is_empty() {
            let custom_meta_map = self
                .custom_metas
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<HashMap<_, _>>();
            req_header_map.extend(custom_meta_map);
        };

        let header_map = get_request_header(client, req_header_map, &request_url, HTTPVerb::Put);

        let data = match object {
            PutObjectBody::Bytes(bytes) => Body::from(bytes),
            PutObjectBody::FilePath(path) => {
                let file = tokio::fs::File::open(path).await?;
                let stream = ReaderStream::new(file);
                Body::wrap_stream(stream)
            }
        };

        let resp = client
            .http_client
            .put(request_url)
            .headers(header_map)
            .body(data)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(into_request_failed_error(resp).await);
        }

        let header = resp.headers();
        let content_md5 = header
            .get("Content-MD5")
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        let x_oss_hash_crc64ecma = header
            .get("x-oss-hash-crc64ecma")
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        let x_oss_version_id = header
            .get("x-oss-version-id")
            .map(|v| v.to_str().unwrap().to_owned());

        Ok(PutObjectResponseHeader {
            content_md5,
            x_oss_hash_crc64ecma,
            x_oss_version_id,
        })
    }
}

impl GetObject<'_> {
    /// 返回：
    /// - `Vec<u8>`：文件数据
    /// - `HashMap<String, String>`：所有响应头
    pub async fn receive_bytes(
        &self,
        object_name: &str,
    ) -> Result<(Bytes, GetObjectResponseHeader, HeaderMap), Error> {
        let (resp, response_header, header) = self.get_response(object_name).await?;
        let data = resp.bytes().await?;

        Ok((data, response_header, header))
    }

    pub async fn receive_bytes_stream(
        &self,
        object_name: &str,
    ) -> Result<
        (
            impl Stream<Item = Result<Bytes, Error>> + use<>,
            GetObjectResponseHeader,
            HeaderMap,
        ),
        Error,
    > {
        let (resp, response_header, header) = self.get_response(object_name).await?;
        let byte_stream = resp.bytes_stream().map(|item| item.map_err(Error::Reqwest));
        Ok((byte_stream, response_header, header))
    }

    pub async fn download_to_file(
        &self,
        object_name: &str,
        file_path: &Path,
    ) -> Result<(GetObjectResponseHeader, HeaderMap), Error> {
        let (mut resp, response_header, header) = self.get_response(object_name).await?;

        let mut file = tokio::fs::File::create(file_path).await?;
        while let Some(chunk) = resp.chunk().await? {
            file.write_all(&chunk).await?;
        }
        file.flush().await?;

        Ok((response_header, header))
    }

    async fn get_response(
        &self,
        object_name: &str,
    ) -> Result<(reqwest::Response, GetObjectResponseHeader, HeaderMap), Error> {
        validate_object_name(object_name)?;

        let client = self.client;
        let request_url = url::Url::parse(&format!(
            "https://{}.{}/{}",
            client.bucket, client.endpoint, object_name
        ))
        .unwrap();

        let req_header_map = serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap();
        let header_map = get_request_header(client, req_header_map, &request_url, HTTPVerb::Get);

        let resp = client
            .http_client
            .get(request_url)
            .headers(header_map)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            return Err(into_request_failed_error(resp).await);
        }

        let header = resp.headers().clone();
        let (mut response_header, custom_meta_map) =
            parse_get_object_response_header::<GetObjectResponseHeader>(&header);
        if !custom_meta_map.is_empty() {
            response_header.custom_x_oss_meta = custom_meta_map;
        }

        Ok((resp, response_header, header))
    }
}

impl CopyObject<'_> {
    /// - 不会对参数的bucket，endpoint，object_name，region进行合法性检查，需要自行保证
    /// - `copy_object_dest_info`：如为None，将使用client中的提供的相关信息
    pub async fn send(
        &self,
        dest_bucket: &str,
        dest_object_name: &str,
    ) -> Result<CopyObjectResult, Error> {
        validate_object_name(dest_object_name)?;

        let client = self.client;
        let request_url = url::Url::parse(&format!(
            "https://{}.{}/{}",
            dest_bucket, client.endpoint, dest_object_name
        ))
        .unwrap();

        let req_header_map = serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap();
        let header_map = get_request_header(client, req_header_map, &request_url, HTTPVerb::Put);

        let resp = client
            .http_client
            .put(request_url)
            .headers(header_map)
            .send()
            .await?;

        let data = parse_xml_response(resp).await?;
        Ok(data)
    }
}

impl AppendObject<'_> {
    /// - 当创建一个新的Appendable Object的时候，`position`设为`0`
    /// - 如果该object已存在，则`position`为该Object的字节大小，即此次append object的起始位置
    pub async fn send(
        &self,
        object_name: &str,
        position: u64,
        data: Vec<u8>,
    ) -> Result<(u64, String), Error> {
        validate_object_name(object_name)?;

        let client = self.client;
        let request_url = url::Url::parse_with_params(
            &format!(
                "https://{}.{}/{}",
                client.bucket, client.endpoint, object_name
            ),
            [("append", ""), ("position", &position.to_string())],
        )
        .unwrap();

        let mut req_header_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap();

        if !self.custom_metas.is_empty() {
            let custom_meta_map = self
                .custom_metas
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<HashMap<_, _>>();
            req_header_map.extend(custom_meta_map);
        }

        req_header_map.insert("content-md5".to_owned(), get_content_md5(&data));
        req_header_map.insert("content-length".to_owned(), data.len().to_string());

        let header_map = get_request_header(client, req_header_map, &request_url, HTTPVerb::Post);

        let resp = client
            .http_client
            .post(request_url)
            .headers(header_map)
            .body(data)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(into_request_failed_error(resp).await);
        }

        let next_position = resp
            .headers()
            .get("x-oss-next-append-position")
            .unwrap()
            .to_str()
            .unwrap()
            .parse::<u64>()
            .unwrap();
        let response_hash = resp
            .headers()
            .get("x-oss-hash-crc64ecma")
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();

        Ok((next_position, response_hash))
    }
}

impl DeleteMultipleObjects<'_> {
    /// 如果`quiet`为`true`，响应体没有内容，返回None
    /// 如果`quiet`为`false`，响应体包含删除结果，返回Some(DeleteResult)
    pub async fn send(&self) -> Result<Option<DeleteResult>, Error> {
        let client = self.client;

        let request_url = url::Url::parse(&format!(
            "https://{}.{}/?delete",
            client.bucket, client.endpoint
        ))
        .unwrap();
        let delete_req = DeleteMultipleObjectsRequest {
            quiet: self.quiet,
            object: &self.objects,
        };
        let req_body = quick_xml::se::to_string_with_root("Delete", &delete_req).unwrap();
        let mut req_header_map = HashMap::with_capacity(3);
        if let Some(encoding_type) = self.encoding_type {
            req_header_map.insert("encoding-type".to_owned(), encoding_type.to_owned());
        }
        req_header_map.insert("content-length".to_owned(), req_body.len().to_string());
        req_header_map.insert(
            "content-md5".to_owned(),
            get_content_md5(req_body.as_bytes()),
        );

        let header_map = get_request_header(client, req_header_map, &request_url, HTTPVerb::Post);

        let resp = client
            .http_client
            .post(request_url)
            .headers(header_map)
            .body(req_body)
            .send()
            .await?;

        // 如果是is_quiet为true的请求，返回的xml中没有删除结果，使用Option来简化处理
        let data = parse_xml_response(resp).await?;
        Ok(data)
    }
}

impl HeadObject<'_> {
    pub async fn send(
        &self,
        object_name: &str,
    ) -> Result<(HeadObjectResponseHeader, HeaderMap), Error> {
        validate_object_name(object_name)?;

        let client = self.client;
        let request_url = url::Url::parse(&format!(
            "https://{}.{}/{}",
            client.bucket, client.endpoint, object_name
        ))
        .unwrap();

        let req_header_map = serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap();
        let header_map = get_request_header(client, req_header_map, &request_url, HTTPVerb::Head);

        let resp = client
            .http_client
            .head(request_url)
            .headers(header_map)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            return Err(into_request_failed_error(resp).await);
        }

        let header = resp.headers().clone();
        let (mut response_header, custom_meta_map) =
            parse_get_object_response_header::<HeadObjectResponseHeader>(&header);
        if !custom_meta_map.is_empty() {
            response_header.custom_x_oss_meta = custom_meta_map;
        }
        Ok((response_header, header))
    }
}

/// Object基础操作
impl Client {
    pub fn put_object(&self) -> PutObjectBuilder<'_> {
        PutObject::builder(self)
    }

    pub fn get_object(&self) -> GetObjectBuilder<'_> {
        GetObject::builder(self)
    }

    pub fn copy_object(&self) -> CopyObjectBuilder<'_> {
        CopyObject::builder(self)
    }

    pub fn append_object(&self) -> AppendObjectBuilder<'_> {
        AppendObject::builder(self)
    }

    /// 无论object是否存在都会执行删除操作并返回成功
    pub async fn delete_object(
        &self,
        object_name: &str,
    ) -> Result<DeleteObjectResponseHeader, Error> {
        validate_object_name(object_name)?;

        let request_url = url::Url::parse(&format!(
            "https://{}.{}/{}",
            self.bucket, self.endpoint, object_name
        ))
        .unwrap();

        let header_map = get_request_header(
            self,
            HashMap::with_capacity(0),
            &request_url,
            HTTPVerb::Delete,
        );

        let resp = self
            .http_client
            .delete(request_url)
            .headers(header_map)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(into_request_failed_error(resp).await);
        }

        let x_oss_delete_marker = resp
            .headers()
            .get("x-oss-delete-marker")
            .map(|v| v.to_str().unwrap().parse::<bool>().unwrap());
        let x_oss_version_id = resp
            .headers()
            .get("x-oss-version-id")
            .map(|v| v.to_str().unwrap().to_owned());

        Ok(DeleteObjectResponseHeader {
            x_oss_delete_marker,
            x_oss_version_id,
        })
    }

    pub fn delete_multiple_objects(&self) -> DeleteMultipleObjectsBuilder<'_> {
        DeleteMultipleObjects::builder(self)
    }

    pub fn head_object(&self) -> HeadObjectBuilder<'_> {
        HeadObject::builder(self)
    }

    /// - 这里返回`HashMap`而没有返回struct，主要考虑到response header中有一些参数文档中没说出来，不便于转化为指定的struct
    /// - 返回的`HashMap`中所有的`key`均为小写，这里代码并没有使用`to_lowercase`，因为`reqwest`获取的header都为小写
    pub async fn get_object_meta(
        &self,
        object_name: &str,
    ) -> Result<GetObjectMetaResponseHeader, Error> {
        validate_object_name(object_name)?;

        let request_url = url::Url::parse(&format!(
            "https://{}.{}/{}?objectMeta",
            self.bucket, self.endpoint, object_name
        ))
        .unwrap();

        let header_map = get_request_header(
            self,
            HashMap::with_capacity(0),
            &request_url,
            HTTPVerb::Head,
        );

        let resp = self
            .http_client
            .head(request_url)
            .headers(header_map)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            return Err(into_request_failed_error(resp).await);
        }

        let mut response_header = Map::with_capacity(10);
        for (name, val) in resp.headers().iter() {
            let name_s = name.as_str();
            if let Ok(s) = val.to_str() {
                response_header.insert(name_s.to_string(), Value::String(s.to_string()));
            }
        }

        let data = serde_json::from_value(Value::Object(response_header)).unwrap();
        Ok(data)
    }
}
