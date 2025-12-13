//! 关于Object操作/基础操作
//!
//! [官方文档](https://help.aliyun.com/zh/oss/developer-reference/basic-operations-1/)

use super::types_rs::*;
use crate::oss::Client;
use crate::oss::Error;
use crate::oss::sign_v4::HTTPVerb;
use crate::oss::utils::{
    PresignParams, compute_md5_from_file, generate_presigned_url, get_content_md5,
    get_request_header, hmac_sha256_bytes, into_request_failed_error,
    parse_get_object_response_header, parse_xml_response, utc_date_str, utc_date_time_str,
    validate_object_name,
};
use base64::{Engine, engine::general_purpose};
use bytes::Bytes;
use reqwest::header::HeaderMap;
use reqwest::{Body, StatusCode};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::Path;
use time::OffsetDateTime;
use time::format_description::well_known::Iso8601;
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

        if let Some(oss_callback) = &self.callback {
            let callback_base64 =
                general_purpose::STANDARD.encode(serde_json::to_string(oss_callback).unwrap());
            // println!("callback_base64: {}", &callback_base64);
            req_header_map.insert("x-oss-callback".to_owned(), callback_base64);
            if !oss_callback.callback_body.callback_var.is_empty() {
                let callback_var_map = oss_callback
                    .callback_body
                    .callback_var
                    .iter()
                    .map(|(_, k, v)| (k, v))
                    .collect::<HashMap<_, _>>();
                let callback_var_base64 = general_purpose::STANDARD
                    .encode(serde_json::to_string(&callback_var_map).unwrap());
                // println!("callback_var_base64: {}", &callback_var_base64);
                req_header_map.insert("x-oss-callback-var".to_owned(), callback_var_base64);
            }
        }

        let creds = client.credentials_provider.load().await?;
        // sts token
        if let Some(token) = &creds.sts_security_token {
            req_header_map.insert("x-oss-security-token".to_owned(), token.clone());
        }

        let header_map = get_request_header(
            &creds.access_key_id,
            &creds.access_key_secret,
            req_header_map,
            &request_url,
            HTTPVerb::Put,
            &client.region,
            Some(&client.bucket),
        );

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
        // println!("response: {:#?}", resp);

        // 如果上传成功，但是使用callback的时候应用服务器端没有响应导致回调失败，会返回203
        if resp.status() != StatusCode::OK {
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

    /// 生成用于上传的预签名URL（Presigned URL），生成的url使用PUT方法上传文件
    ///
    /// [在URL中包含签名](https://help.aliyun.com/zh/oss/developer-reference/add-signatures-to-urls)
    ///
    /// [OSS不直接提供限制上传文件类型和大小的功能](https://help.aliyun.com/zh/oss/how-do-i-limit-object-formats-and-sizes-when-i-upload-objects-to-oss)
    ///
    /// # 参数
    /// - `object_name`：要上传的对象名称。必须遵守 [OSS Object 命名规则](https://help.aliyun.com/zh/oss/user-guide/object-overview#720fde5f0asvg)。
    /// - `expires`：URL 的有效期，单位为秒。过期后将无法使用。
    ///
    /// 阿里云oss文档不建议使用预签名URL上传带有回调的对象：
    /// > 该方式常用于预签名URL上传文件的场景，通过将回调参数Base64编码后拼接在URL中实现自动回调。
    /// > 但由于回调信息暴露在 URL 中，存在一定的安全风险，仅建议用于临时访问或低敏感场景。[callback签名](https://help.aliyun.com/zh/oss/developer-reference/callback)
    pub async fn generate_presigned_url(
        &self,
        object_name: &str,
        expires: i32,
    ) -> Result<String, Error> {
        validate_object_name(object_name)?;

        let client = self.client;
        let mut base_url = url::Url::parse(&format!(
            "https://{}.{}/{}",
            client.bucket, client.endpoint, object_name
        ))
        .unwrap();

        // callback处理
        if let Some(oss_callback) = &self.callback {
            let callback_base64 =
                general_purpose::STANDARD.encode(serde_json::to_string(oss_callback).unwrap());
            base_url
                .query_pairs_mut()
                .append_pair("callback", &callback_base64);
            if !oss_callback.callback_body.callback_var.is_empty() {
                let callback_var_map = oss_callback
                    .callback_body
                    .callback_var
                    .iter()
                    .map(|(_, k, v)| (k, v))
                    .collect::<HashMap<_, _>>();
                let callback_var_base64 = general_purpose::STANDARD
                    .encode(serde_json::to_string(&callback_var_map).unwrap());
                base_url
                    .query_pairs_mut()
                    .append_pair("callback-var", &callback_var_base64);
            }
        }

        let creds = client.credentials_provider.load().await?;
        // sts token
        if let Some(token) = &creds.sts_security_token {
            base_url
                .query_pairs_mut()
                .append_pair("x-oss-security-token", token);
        }

        let mut header_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap();
        if !self.custom_metas.is_empty() {
            let custom_meta_map = self
                .custom_metas
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<HashMap<_, _>>();
            header_map.extend(custom_meta_map);
        };

        let presigned_params = PresignParams {
            access_key_id: &creds.access_key_id,
            access_key_secret: &creds.access_key_secret,
            header_map,
            presigned_url: base_url,
            http_verb: HTTPVerb::Put,
            url_expires: expires,
            bucket: &client.bucket,
            signing_region: &client.region,
        };
        let signed_url = generate_presigned_url(presigned_params);
        Ok(signed_url)
    }
}

impl PostObject<'_> {
    /// 生成用于浏览器表单方式上传所需要的内容
    ///
    /// - [oss Post v4 签名文档](https://help.aliyun.com/zh/oss/developer-reference/signature-version-4-recommend)
    /// - [PostObject API文档](https://help.aliyun.com/zh/oss/developer-reference/postobject)
    ///
    /// # 参数
    /// - `expiration`：策略过期时间
    ///
    /// # 注意
    /// 当你使用sts的方式生成policy后。在发起表单请求的时候请求头如果没有携带`x-oss-security-token`，会报`InvalidAccessKeyId`的错误
    pub async fn generate_policy(
        self,
        expiration: OffsetDateTime,
    ) -> Result<GeneratePolicyResult, Error> {
        let policy_expiration = expiration.to_utc().format(&Iso8601::DEFAULT).unwrap();
        let now = OffsetDateTime::now_utc();
        // 这个date，需不需utc，文档没说...
        let date = utc_date_str(&now);
        let date_time = utc_date_time_str(&now);
        let client = self.client;
        let creds = client.credentials_provider.load().await?;
        let credential = format!(
            "{}/{}/{}/oss/aliyun_v4_request",
            creds.access_key_id, date, client.region
        );

        // 处理callback相关
        let mut callback_b64 = None;
        let mut callback_var = None;
        if let Some(oss_callback) = &self.callback {
            callback_b64 = Some(
                general_purpose::STANDARD.encode(serde_json::to_string(oss_callback).unwrap()),
            );
            if !oss_callback.callback_body.callback_var.is_empty() {
                let callback_var_map = oss_callback
                    .callback_body
                    .callback_var
                    .iter()
                    .map(|(_, k, v)| (k.clone(), v.clone()))
                    .collect::<HashMap<_, _>>();
                callback_var = Some(callback_var_map);
            }
        }

        let policy = PostPolicy {
            expiration: policy_expiration,
            conditions: PostPolicyCondition {
                bucket: self.bucket,
                x_oss_signature_version: "OSS4-HMAC-SHA256".to_owned(),
                x_oss_credential: credential.clone(),
                x_oss_security_token: creds.sts_security_token.clone(),
                x_oss_date: date_time.clone(),
                content_length_range: self.content_length_range,
                key: self.key,
                success_action_status: self.success_action_status,
                content_type: self.content_type,
                cache_control: self.cache_control,
                expires: self.expires,
                content_disposition: self.content_disposition,
                content_encoding: self.content_encoding,
                x_oss_object_acl: self.x_oss_object_acl,
                x_oss_server_side_encryption_key_id: self.x_oss_server_side_encryption_key_id,
                x_oss_server_side_data_encryption: self.x_oss_server_side_data_encryption,
                x_oss_content_type: self.x_oss_content_type,
                x_oss_forbid_overwrite: self.x_oss_forbid_overwrite,
                x_oss_storage_class: self.x_oss_storage_class,
                success_action_redirect: self.success_action_redirect,
                custom_metas: self.custom_metas,
                callback_b64: callback_b64.as_deref(),
                callback_var: callback_var.as_ref(),
            },
        };
        let policy_str = serde_json::to_string(&policy).unwrap();
        let encoded_policy = general_purpose::STANDARD.encode(policy_str.as_bytes());
        let date_key = hmac_sha256_bytes(
            format!("aliyun_v4{}", creds.access_key_secret).as_bytes(),
            &date,
        );
        let date_region_key = hmac_sha256_bytes(&date_key, &client.region);
        let date_region_service_key = hmac_sha256_bytes(&date_region_key, "oss");
        let signing_key = hmac_sha256_bytes(&date_region_service_key, "aliyun_v4_request");
        let signature = hex::encode(hmac_sha256_bytes(&signing_key, &encoded_policy));

        Ok(GeneratePolicyResult {
            policy: encoded_policy,
            signature,
            date_time,
            credential,
            callback_b64,
            callback_var,
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

    /// 生成预签名URL
    ///
    /// - `expires`：URL过期时间，单位秒
    ///
    /// 使用长期访问密钥AccessKey生成签名URL，该字段取值要求：最小值为 1 秒，最大值为 604800秒（ 7 天）。
    ///
    /// 使用STS临时访问凭证生成签名URL，该字段取值要求：最小值为 1 秒，最大有效时长为 43200秒（ 12 小时）。
    ///
    /// [签名文档和说明](https://help.aliyun.com/zh/oss/developer-reference/add-signatures-to-urls)
    pub async fn generate_presigned_url(
        &self,
        object_name: &str,
        expires: i32,
    ) -> Result<String, Error> {
        validate_object_name(object_name)?;

        let client = self.client;
        let mut base_url = url::Url::parse(&format!(
            "https://{}.{}/{}",
            client.bucket, client.endpoint, object_name
        ))
        .unwrap();
        // 先把所有query参数添加到url中，这样在签名的时候直接传递url即可获取所有query参数
        let query_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(self.queries_part()).unwrap()).unwrap();
        for (k, v) in query_map.iter() {
            base_url.query_pairs_mut().append_pair(k, v);
        }
        let creds = client.credentials_provider.load().await?;
        if let Some(token) = &creds.sts_security_token {
            base_url
                .query_pairs_mut()
                .append_pair("x-oss-security-token", token);
        }

        let header_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(self.headers_part()).unwrap()).unwrap();
        let presigned_params = PresignParams {
            access_key_id: &creds.access_key_id,
            access_key_secret: &creds.access_key_secret,
            header_map,
            presigned_url: base_url,
            http_verb: HTTPVerb::Get,
            url_expires: expires,
            bucket: &client.bucket,
            signing_region: &client.region,
        };
        let signed_url = generate_presigned_url(presigned_params);
        Ok(signed_url)
    }

    async fn get_response(
        &self,
        object_name: &str,
    ) -> Result<(reqwest::Response, GetObjectResponseHeader, HeaderMap), Error> {
        validate_object_name(object_name)?;

        let client = self.client;
        let mut request_url = url::Url::parse(&format!(
            "https://{}.{}/{}",
            client.bucket, client.endpoint, object_name
        ))
        .unwrap();
        let query_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(self.queries_part()).unwrap()).unwrap();
        for (k, v) in query_map.iter() {
            request_url.query_pairs_mut().append_pair(k, v);
        }

        let mut req_header_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(self.headers_part()).unwrap()).unwrap();
        let creds = client.credentials_provider.load().await?;
        if let Some(token) = &creds.sts_security_token {
            req_header_map.insert("x-oss-security-token".to_owned(), token.clone());
        }
        let header_map = get_request_header(
            &creds.access_key_id,
            &creds.access_key_secret,
            req_header_map,
            &request_url,
            HTTPVerb::Get,
            &client.region,
            Some(&client.bucket),
        );

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

        let mut req_header_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap();
        let creds = client.credentials_provider.load().await?;
        if let Some(token) = &creds.sts_security_token {
            req_header_map.insert("x-oss-security-token".to_owned(), token.clone());
        }
        let header_map = get_request_header(
            &creds.access_key_id,
            &creds.access_key_secret,
            req_header_map,
            &request_url,
            HTTPVerb::Put,
            &client.region,
            Some(&client.bucket),
        );

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

        let creds = client.credentials_provider.load().await?;
        if let Some(token) = &creds.sts_security_token {
            req_header_map.insert("x-oss-security-token".to_owned(), token.clone());
        }

        let header_map = get_request_header(
            &creds.access_key_id,
            &creds.access_key_secret,
            req_header_map,
            &request_url,
            HTTPVerb::Post,
            &client.region,
            Some(&client.bucket),
        );

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
        let mut req_header_map = HashMap::new();
        if let Some(encoding_type) = self.encoding_type {
            req_header_map.insert("encoding-type".to_owned(), encoding_type.to_owned());
        }
        req_header_map.insert("content-length".to_owned(), req_body.len().to_string());
        req_header_map.insert(
            "content-md5".to_owned(),
            get_content_md5(req_body.as_bytes()),
        );

        let creds = client.credentials_provider.load().await?;
        if let Some(token) = &creds.sts_security_token {
            req_header_map.insert("x-oss-security-token".to_owned(), token.clone());
        }

        let header_map = get_request_header(
            &creds.access_key_id,
            &creds.access_key_secret,
            req_header_map,
            &request_url,
            HTTPVerb::Post,
            &client.region,
            Some(&client.bucket),
        );

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

        let mut req_header_map: HashMap<String, String> =
            serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap();
        let creds = client.credentials_provider.load().await?;
        if let Some(token) = &creds.sts_security_token {
            req_header_map.insert("x-oss-security-token".to_owned(), token.clone());
        }

        let header_map = get_request_header(
            &creds.access_key_id,
            &creds.access_key_secret,
            req_header_map,
            &request_url,
            HTTPVerb::Head,
            &client.region,
            Some(&client.bucket),
        );

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

    pub fn post_object(&self) -> PostObjectBuilder<'_> {
        PostObject::builder(self)
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

        let creds = self.credentials_provider.load().await?;
        let mut req_header_map = HashMap::new();
        if let Some(token) = &creds.sts_security_token {
            req_header_map.insert("x-oss-security-token".to_owned(), token.clone());
        }

        let header_map = get_request_header(
            &creds.access_key_id,
            &creds.access_key_secret,
            req_header_map,
            &request_url,
            HTTPVerb::Delete,
            &self.region,
            Some(&self.bucket),
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

        let creds = self.credentials_provider.load().await?;
        let mut req_header_map = HashMap::new();
        if let Some(token) = &creds.sts_security_token {
            req_header_map.insert("x-oss-security-token".to_owned(), token.clone());
        }
        let header_map = get_request_header(
            &creds.access_key_id,
            &creds.access_key_secret,
            req_header_map,
            &request_url,
            HTTPVerb::Head,
            &self.region,
            Some(&self.bucket),
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

        let mut response_header = Map::new();
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
