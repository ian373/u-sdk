//! 关于Object操作/基础操作
//!
//! [官方文档](https://help.aliyun.com/zh/oss/developer-reference/basic-operations-1/)

use super::types_rs::*;
use super::utils::partition_header;
use crate::error::Error;
use crate::oss::OSSClient;
use crate::oss::sign_v4::{HTTPVerb, SignV4Param};
use crate::oss::utils::{
    SerializeToHashMap, get_content_md5, handle_response_status, into_request_header,
};
use common_lib::helper::gmt_format;

use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Object基础操作
impl OSSClient {
    /// - `content_type`，不会进行MIME合法性检查
    /// - `object_name`：不会进行合法性检查，遵守OSS的Object命名规则
    /// - `data`：如果需要创建文件夹，object_name以`/`结尾，`Vec`大小为0即可
    ///
    /// 上传成功后不会返回响应内容
    pub async fn put_object(
        &self,
        put_bucket_header: PutObjectHeader<'_>,
        x_meta_header: Option<XMetaHeader<'_>>,
        object_name: &str,
        data: Vec<u8>,
    ) -> Result<(), Error> {
        let request_url = url::Url::parse(&format!(
            "https://{}.{}/{}", // url不能添加`/`结尾，因为是否有`/`由object_name决定
            self.bucket, self.endpoint, object_name
        ))
        .unwrap();

        let mut req_header_map = put_bucket_header.serialize_to_hashmap()?;
        // 添加api剩下的请求头
        req_header_map.insert("content-md5".to_owned(), get_content_md5(&data));
        req_header_map.insert("content-length".to_owned(), data.len().to_string());
        // 把需要签名的header和不需要签名的header分开
        let (sign_map, remaining_map) = partition_header(req_header_map);

        // 创建CanonicalHeaders，把所有需要签名的header放到CanonicalHeaders中
        let mut canonical_header = BTreeMap::new();
        canonical_header.extend(sign_map.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        // 如果有x_meta_header，将其添加到canonical_header中参与签名
        let meta_map = if let Some(m) = x_meta_header {
            m.get_meta_map()
        } else {
            HashMap::new()
        };
        canonical_header.extend(meta_map.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        canonical_header.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD");
        canonical_header.insert("host", request_url.host_str().unwrap());

        let mut additional_header = BTreeSet::new();
        additional_header.insert("host");
        let now = time::OffsetDateTime::now_utc();
        let sign_v4_param = SignV4Param {
            signing_region: &self.region,
            http_verb: HTTPVerb::Put,
            uri: &request_url,
            bucket: Some(&self.bucket),
            header_map: &canonical_header,
            additional_header: Some(&additional_header),
            date_time: &now,
        };
        let authorization = self.sign_v4(sign_v4_param);

        // 把canonical_header转化为最终的header，补齐剩下的未参与签名计算的header
        // 包括：剩下必要的公共请求头，api header中的非签名字段
        let mut header = canonical_header.into_iter().collect::<HashMap<_, _>>();
        header.insert("Authorization", &authorization);
        let gmt = gmt_format(&now);
        header.insert("Date", &gmt);
        header.extend(remaining_map.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        let header_map = into_request_header(header);

        let resp = self
            .http_client
            .put(request_url)
            .headers(header_map)
            .body(data)
            .send()
            .await?;

        let _ = handle_response_status(resp).await?;

        Ok(())
    }

    /// 返回：
    /// - `Vec<u8>`：文件数据
    /// - `HashMap<String, String>`：所有响应头
    pub async fn get_object(
        &self,
        get_object_header: GetObjectHeader<'_>,
        object_name: &str,
    ) -> Result<(Vec<u8>, HashMap<String, String>), Error> {
        let request_url = url::Url::parse(&format!(
            "https://{}.{}/{}",
            self.bucket, self.endpoint, object_name
        ))
        .unwrap();

        let mut canonical_header = BTreeMap::new();
        canonical_header.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD");
        canonical_header.insert("host", request_url.host_str().unwrap());

        let mut additional_header = BTreeSet::new();
        additional_header.insert("host");
        let now = time::OffsetDateTime::now_utc();
        let sign_v4_param = SignV4Param {
            signing_region: &self.region,
            http_verb: HTTPVerb::Get,
            uri: &request_url,
            bucket: Some(&self.bucket),
            header_map: &canonical_header,
            additional_header: Some(&additional_header),
            date_time: &now,
        };
        let authorization = self.sign_v4(sign_v4_param);

        let mut header = canonical_header.into_iter().collect::<HashMap<_, _>>();
        let req_header_map = get_object_header.serialize_to_hashmap()?;
        header.extend(req_header_map.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        header.insert("Authorization", &authorization);
        let gmt = gmt_format(&now);
        header.insert("Date", &gmt);
        let header_map = into_request_header(header);

        let resp = self
            .http_client
            .get(request_url)
            .headers(header_map)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await?;
            return Err(Error::RequestAPIFailed {
                status: status.to_string(),
                text,
            });
        }
        let resp_header = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_owned()))
            .collect();
        let data = resp.bytes().await?.to_vec();

        Ok((data, resp_header))
    }

    /// - 不会对参数的bucket，endpoint，object_name，region进行合法性检查，需要自行保证
    /// - `copy_object_dest_info`：如为None，将使用client中的提供的相关信息
    pub async fn copy_object(
        &self,
        copy_object_x_header: CopyObjectXHeader<'_>,
        dest_object_name: &str,
        copy_object_dest_info: Option<CopyObjectDestInfo<'_>>,
    ) -> Result<(), Error> {
        let (dest_region, dest_end_point, dest_bucket) =
            if let Some(dest_info) = copy_object_dest_info {
                (dest_info.region, dest_info.endpoint, dest_info.bucket)
            } else {
                (
                    self.region.as_ref(),
                    self.endpoint.as_ref(),
                    self.bucket.as_ref(),
                )
            };
        let request_url = url::Url::parse(&format!(
            "https://{}.{}/{}",
            dest_bucket, dest_end_point, dest_object_name
        ))
        .unwrap();
        let mut canonical_header = BTreeMap::new();
        let copy_object_header = copy_object_x_header.serialize_to_hashmap()?;
        canonical_header.extend(
            copy_object_header
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str())),
        );
        canonical_header.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD");
        canonical_header.insert("host", request_url.host_str().unwrap());

        let mut additional_header = BTreeSet::new();
        additional_header.insert("host");
        let now = time::OffsetDateTime::now_utc();
        let sign_v4_param = SignV4Param {
            signing_region: dest_region,
            http_verb: HTTPVerb::Put,
            uri: &request_url,
            bucket: Some(dest_bucket),
            header_map: &canonical_header,
            additional_header: Some(&additional_header),
            date_time: &now,
        };
        let authorization = self.sign_v4(sign_v4_param);

        let mut header = canonical_header.into_iter().collect::<HashMap<_, _>>();
        header.insert("Authorization", &authorization);
        let gmt = gmt_format(&now);
        header.insert("Date", &gmt);
        let header_map = into_request_header(header);

        let resp = self
            .http_client
            .put(request_url)
            .headers(header_map)
            .send()
            .await?;

        let _ = handle_response_status(resp).await?;

        Ok(())
    }

    /// - 当创建一个新的Appendable Object的时候，`position`设为`0`
    /// - 如果该object已存在，则`position`为该Object的字节大小，即此次append object的起始位置
    pub async fn append_object(
        &self,
        object_name: &str,
        append_object_header: AppendObjectHeader<'_>,
        x_meta_header: Option<XMetaHeader<'_>>,
        data: Vec<u8>,
    ) -> Result<u64, Error> {
        let request_url = url::Url::parse_with_params(
            &format!("https://{}.{}/{}", self.bucket, self.endpoint, object_name),
            [
                ("append", ""),
                ("position", &append_object_header.position.to_string()),
            ],
        )
        .unwrap();
        let mut req_header_map = append_object_header.serialize_to_hashmap()?;
        req_header_map.insert("content-md5".to_owned(), get_content_md5(&data));
        req_header_map.insert("content-length".to_owned(), data.len().to_string());
        let (sign_map, remaining_map) = partition_header(req_header_map);

        let mut canonical_header = BTreeMap::new();
        canonical_header.extend(sign_map.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        let meta_map = if let Some(m) = x_meta_header {
            m.get_meta_map()
        } else {
            HashMap::with_capacity(0)
        };
        canonical_header.extend(meta_map.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        canonical_header.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD");
        canonical_header.insert("host", request_url.host_str().unwrap());

        let mut additional_header = BTreeSet::new();
        additional_header.insert("host");
        let now = time::OffsetDateTime::now_utc();
        let sign_v4_param = SignV4Param {
            signing_region: &self.region,
            http_verb: HTTPVerb::Post,
            uri: &request_url,
            bucket: Some(&self.bucket),
            header_map: &canonical_header,
            additional_header: Some(&additional_header),
            date_time: &now,
        };
        let authorization = self.sign_v4(sign_v4_param);

        let mut header = canonical_header.into_iter().collect::<HashMap<_, _>>();
        header.insert("Authorization", &authorization);
        let gmt = gmt_format(&now);
        header.insert("Date", &gmt);
        header.extend(remaining_map.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        let header_map = into_request_header(header);

        let mut resp = self
            .http_client
            .post(request_url)
            .headers(header_map)
            .body(data)
            .send()
            .await?;

        let next_position = resp.headers_mut().remove("x-oss-next-append-position");
        let _ = handle_response_status(resp).await?;
        let next_position = next_position
            .unwrap()
            .to_str()
            .unwrap()
            .parse::<u64>()
            .unwrap();

        Ok(next_position)
    }

    /// 无论object是否存在都会执行删除操作并返回成功
    pub async fn delete_object(&self, object_name: &str) -> Result<(), Error> {
        let request_url = url::Url::parse(&format!(
            "https://{}.{}/{}",
            self.bucket, self.endpoint, object_name
        ))
        .unwrap();

        let mut canonical_header = BTreeMap::new();
        canonical_header.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD");
        canonical_header.insert("host", request_url.host_str().unwrap());

        let mut additional_header = BTreeSet::new();
        additional_header.insert("host");
        let now = time::OffsetDateTime::now_utc();
        let sign_v4_param = SignV4Param {
            signing_region: &self.region,
            http_verb: HTTPVerb::Delete,
            uri: &request_url,
            bucket: Some(&self.bucket),
            header_map: &canonical_header,
            additional_header: Some(&additional_header),
            date_time: &now,
        };
        let authorization = self.sign_v4(sign_v4_param);

        let mut header = canonical_header.into_iter().collect::<HashMap<_, _>>();
        header.insert("Authorization", &authorization);
        let gmt = gmt_format(&now);
        header.insert("Date", &gmt);
        let header_map = into_request_header(header);

        let resp = self
            .http_client
            .delete(request_url)
            .headers(header_map)
            .send()
            .await?;

        let _ = handle_response_status(resp).await?;
        Ok(())
    }

    pub async fn delete_multiple_objects(
        &self,
        encoding_type: Option<&str>,
        delete_objects: Vec<DeleteObject<'_>>,
        is_quiet_resp: bool,
    ) -> Result<Option<DeleteResult>, Error> {
        let request_url = url::Url::parse(&format!(
            "https://{}.{}/?delete",
            self.bucket, self.endpoint
        ))
        .unwrap();
        let delete_req = DeleteMultipleObjectsRequest {
            quiet: is_quiet_resp,
            object: delete_objects,
        };
        let req_body = quick_xml::se::to_string_with_root("Delete", &delete_req)
            .map_err(|_| Error::AnyError("to string with root error".to_owned()))?;

        let mut canonical_header = BTreeMap::new();
        canonical_header.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD");
        canonical_header.insert("host", request_url.host_str().unwrap());
        let content_md5 = get_content_md5(req_body.as_bytes());
        canonical_header.insert("content-md5", &content_md5);

        let mut additional_header = BTreeSet::new();
        additional_header.insert("host");
        let now = time::OffsetDateTime::now_utc();
        let sign_v4_param = SignV4Param {
            signing_region: &self.region,
            http_verb: HTTPVerb::Post,
            uri: &request_url,
            bucket: Some(&self.bucket),
            header_map: &canonical_header,
            additional_header: Some(&additional_header),
            date_time: &now,
        };
        let authorization = self.sign_v4(sign_v4_param);

        let mut header = canonical_header.into_iter().collect::<HashMap<_, _>>();
        header.insert("Authorization", &authorization);
        let gmt = gmt_format(&now);
        header.insert("Date", &gmt);
        let content_length = req_body.len().to_string();
        header.insert("Content-Length", &content_length);
        if let Some(encoding) = encoding_type {
            header.insert("Encoding-type", encoding);
        }
        let header_map = into_request_header(header);

        let resp = self
            .http_client
            .post(request_url)
            .headers(header_map)
            .body(req_body)
            .send()
            .await?;

        let text = handle_response_status(resp).await?;
        let res = quick_xml::de::from_str(&text)?;
        Ok(res)
    }

    pub async fn head_object(
        &self,
        object_name: &str,
        head_object_header: Option<HeadObjectHeader<'_>>,
    ) -> Result<HashMap<String, String>, Error> {
        let request_url = url::Url::parse(&format!(
            "https://{}.{}/{}",
            self.bucket, self.endpoint, object_name
        ))
        .unwrap();

        let mut canonical_header = BTreeMap::new();
        canonical_header.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD");
        canonical_header.insert("host", request_url.host_str().unwrap());

        let mut additional_header = BTreeSet::new();
        additional_header.insert("host");
        let now = time::OffsetDateTime::now_utc();
        let sign_v4_param = SignV4Param {
            signing_region: &self.region,
            http_verb: HTTPVerb::Head,
            uri: &request_url,
            bucket: Some(&self.bucket),
            header_map: &canonical_header,
            additional_header: Some(&additional_header),
            date_time: &now,
        };
        let authorization = self.sign_v4(sign_v4_param);

        let mut header = canonical_header.into_iter().collect::<HashMap<_, _>>();
        let req_header_map = if let Some(h) = head_object_header {
            h.serialize_to_hashmap()?
        } else {
            HashMap::with_capacity(0)
        };
        header.extend(req_header_map.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        header.insert("Authorization", &authorization);
        let gmt = gmt_format(&now);
        header.insert("Date", &gmt);
        let header_map = into_request_header(header);

        let resp = self
            .http_client
            .head(request_url)
            .headers(header_map)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await?;
            return Err(Error::RequestAPIFailed {
                status: status.to_string(),
                text,
            });
        }
        let response_header = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_owned()))
            .collect();

        Ok(response_header)
    }

    /// - 这里返回`HashMap`而没有返回struct，主要考虑到response header中有一些参数文档中没说出来，不便于转化为指定的struct
    /// - 返回的`HashMap`中所有的`key`均为小写，这里代码并没有使用`to_lowercase`，因为`reqwest`获取的header都为小写
    pub async fn get_object_meta(
        &self,
        object_name: &str,
    ) -> Result<HashMap<String, String>, Error> {
        let request_url = url::Url::parse(&format!(
            "https://{}.{}/{}?objectMeta",
            self.bucket, self.endpoint, object_name
        ))
        .unwrap();

        let mut canonical_header = BTreeMap::new();
        canonical_header.insert("x-oss-content-sha256", "UNSIGNED-PAYLOAD");
        canonical_header.insert("host", request_url.host_str().unwrap());

        let mut additional_header = BTreeSet::new();
        additional_header.insert("host");
        let now = time::OffsetDateTime::now_utc();
        let sign_v4_param = SignV4Param {
            signing_region: &self.region,
            http_verb: HTTPVerb::Head,
            uri: &request_url,
            bucket: Some(&self.bucket),
            header_map: &canonical_header,
            additional_header: Some(&additional_header),
            date_time: &now,
        };
        let authorization = self.sign_v4(sign_v4_param);

        let mut header = canonical_header.into_iter().collect::<HashMap<_, _>>();
        header.insert("Authorization", &authorization);
        let gmt = gmt_format(&now);
        header.insert("Date", &gmt);
        let header_map = into_request_header(header);

        let resp = self
            .http_client
            .head(request_url)
            .headers(header_map)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await?;
            return Err(Error::RequestAPIFailed {
                status: status.to_string(),
                text,
            });
        }
        let response_header = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_owned()))
            .collect();

        Ok(response_header)
    }
}
