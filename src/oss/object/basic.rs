//! 关于Object操作/基础操作
//!
//! [官方文档](https://help.aliyun.com/zh/oss/developer-reference/basic-operations-1/)

use super::types_rs::*;
use super::utils::partition_header;
use crate::error::Error;
use crate::oss::sign_v4::{HTTPVerb, SignV4Param};
use crate::oss::utils::{
    get_content_md5, handle_response_status, into_request_header, sign_authorization,
    SerializeToHashMap,
};
use crate::oss::OSSClient;
use crate::utils::common::{gmt_format, into_header_map, now_gmt};

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
        // x_meta_header: Option<XMetaHeader>,
    ) -> Result<AppendObjectResponseHeaderInfo, Error> {
        let mut header_map = HashMap::new();
        let c_header_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(c_header).unwrap()).unwrap();
        header_map.extend(c_header_map);

        let x_header_map: BTreeMap<String, String> =
            serde_json::from_value(serde_json::to_value(x_header).unwrap()).unwrap();
        // if let Some(m) = x_meta_header {
        //     x_header_map.append(&mut m.get_btree_map());
        // }

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
        let resp = builder.send().await?;

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
        builder.send().await?;

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

        if quiet_resp {
            return Ok(None);
        }

        let text = resp.text().await?;
        let res = quick_xml::de::from_str(&text)?;

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
        let response_header: HashMap<String, String> = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_owned()))
            .collect();

        Ok(response_header)
    }
}
