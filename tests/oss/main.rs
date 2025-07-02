use oss::object::types_rs::*;
use std::collections::HashMap;
use u_sdk::oss;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AliConfig {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub endpoint: String,
    pub bucket_name: String,
    pub region: String,
}

impl AliConfig {
    pub fn get_conf() -> Self {
        let file_str = std::fs::read_to_string("tests/oss/config.toml").unwrap();
        let conf = toml::from_str(&file_str).unwrap();

        conf
    }
}

fn get_oss_client() -> oss::OSSClient {
    let conf = AliConfig::get_conf();
    oss::OSSClient::new(
        &conf.access_key_id,
        &conf.access_key_secret,
        &conf.endpoint,
        &conf.region,
        &conf.bucket_name,
    )
}

#[tokio::test]
async fn list_buckets_test() {
    let client = get_oss_client();

    let query = oss::service::ListBucketsQueryParams {
        prefix: Some("test"),
        ..Default::default()
    };
    let res = client.list_buckets(None, Some(query)).await;
    match res {
        Ok(s) => println!("res:\n{:#?}", s),
        Err(e) => println!("{}", e),
    }
}

#[tokio::test]
async fn describe_regions_test() {
    let client = get_oss_client();
    let res = client.describe_regions(Some("oss-ap-northeast-1")).await;
    match res {
        Ok(s) => println!("res:\n{:#?}", s),
        Err(e) => println!("{}", e),
    }
}

#[tokio::test]
async fn put_bucket_test() {
    let client = get_oss_client();

    let res = client
        .put_bucket(
            "example-oss-test-todel",
            "oss-cn-hangzhou.aliyuncs.com",
            None,
            None,
        )
        .await;

    match res {
        Ok(_) => println!("success!"),
        Err(e) => println!("{}", e),
    }
}

#[tokio::test]
async fn list_objects_v2_test() {
    use oss::bucket::ListObjectsV2Query;

    let client = get_oss_client();

    let params = ListObjectsV2Query {
        prefix: Some("test/"),
        ..Default::default()
    };
    let res = client.list_objects_v2(params).await;

    match res {
        Ok(s) => println!("res:\n {:#?}", s),
        Err(e) => println!("error: {}", e),
    }
}

#[tokio::test]
async fn get_bucket_info_test() {
    let client = get_oss_client();
    let res = client.get_bucket_info().await;

    match res {
        Ok(s) => println!("res:\n {:#?}", s),
        Err(e) => println!("error: {}", e),
    }
}

#[tokio::test]
async fn get_bucket_location_test() {
    let client = get_oss_client();

    let res = client.get_bucket_location().await;

    match res {
        Ok(s) => println!("res:\n {}", s),
        Err(e) => println!("error: {}", e),
    }
}

#[tokio::test]
async fn get_bucket_stat_test() {
    let client = get_oss_client();

    let res = client.get_bucket_stat().await;

    match res {
        Ok(s) => println!("res:\n {:#?}", s),
        Err(e) => println!("error: {}", e),
    }
}

#[tokio::test]
async fn put_object_test() {
    let data = b"test123".to_vec();
    let client = get_oss_client();
    let put_object_header = PutObjectHeader {
        content_type: Some("text/plain"),
        cache_control: Some("max-age=6666"),
        ..Default::default()
    };
    let mut meta_map = HashMap::new();
    meta_map.insert("key", "value");
    meta_map.insert("key2", "value2");
    let res = client
        .put_object(
            put_object_header,
            Some(XMetaHeader(meta_map)),
            "test/test_txt.txt",
            data,
        )
        .await;
    match res {
        Ok(_) => println!("res: success"),
        Err(e) => println!("error: {}", e),
    }
}

#[tokio::test]
async fn get_object_test() {
    let client = get_oss_client();

    let c_header = GetObjectHeader {
        range: Some("bytes=0-4"),
        ..Default::default()
    };

    let res = client.get_object(c_header, "test/test_txt.txt").await;

    match res {
        Ok((_data, header)) => {
            println!("response_header:{:#?}", header);
        }
        Err(e) => println!("error: {}", e),
    }
}

#[tokio::test]
async fn copy_object_test() {
    let client = get_oss_client();

    let x_header = CopyObjectXHeader {
        x_oss_forbid_overwrite: Some("true"),
        x_oss_copy_source: &format!("/{}/{}", client.bucket(), "test/test_txt.txt"),
        ..Default::default()
    };
    let dest_info = CopyObjectDestInfo {
        region: client.region(),
        bucket: "example-oss-todel",
        endpoint: client.endpoint(),
    };

    let res = client
        .copy_object(x_header, "copy/test_txt_copy.txt", Some(dest_info))
        .await;

    match res {
        Ok(_) => {
            println!("success!");
        }
        Err(e) => println!("error: {}", e),
    }
}

#[tokio::test]
async fn append_object_test() {
    let client = get_oss_client();

    let append_header = AppendObjectHeader {
        content_type: Some("text/plain"),
        position: 13,
        ..Default::default()
    };
    let res = client
        .append_object(
            "test/append_object.txt",
            append_header,
            None,
            b"text123dfasdf".to_vec(),
        )
        .await;

    match res {
        Ok(next_pos) => {
            println!("next_pos:{}", next_pos);
        }
        Err(e) => println!("error: {}", e),
    }
}

#[tokio::test]
async fn delete_object_test() {
    let client = get_oss_client();

    let res = client.delete_object("test/append_object.txt").await;

    match res {
        Ok(_) => println!("delete success!"),
        Err(e) => println!("error: {}", e),
    }
}

#[tokio::test]
async fn delete_multiple_objects_test() {
    let client = get_oss_client();
    let objs = vec![
        DeleteObject {
            key: "test_dir/123.TXT",
            version_id: None,
        },
        DeleteObject {
            key: "test_file/1234567890.txt",
            version_id: None,
        },
    ];
    let res = client.delete_multiple_objects(None, objs, true).await;
    match res {
        Ok(s) => println!("ok_res:{:#?}", s),
        Err(e) => println!("error: {}", e),
    }
}

#[tokio::test]
async fn head_object_test() {
    let client = get_oss_client();

    let res = client.head_object("copy/test_txt_copy.txt", None).await;

    match res {
        Ok(s) => println!("ok_res:{:#?}", s),
        Err(e) => println!("error: {}", e),
    }
}

#[tokio::test]
async fn get_object_meta_test() {
    let client = get_oss_client();

    let res = client.get_object_meta("copy/test_txt_copy.txt").await;

    match res {
        Ok(s) => println!("ok_res:{:#?}", s),
        Err(e) => println!("error: {}", e),
    }
}
