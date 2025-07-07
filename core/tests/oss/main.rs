use futures_util::StreamExt;
use oss::object::types_rs::*;
use std::path::Path;
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

fn get_oss_client() -> oss::Client {
    let conf = AliConfig::get_conf();
    oss::Client::builder()
        .access_key_id(conf.access_key_id)
        .access_key_secret(conf.access_key_secret)
        .endpoint(conf.endpoint)
        .region(conf.region)
        .bucket(conf.bucket_name)
        .build()
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
    let client = get_oss_client();

    let res = client
        .put_object()
        .content_type("text/plain")
        .cache_control("max-age=6666")
        .x_meta("key", "value")
        .x_metas([("key3", "value3"), ("key4", "value4")])
        .build()
        .send(
            "/test/sample.toml",
            PutObjectBody::FilePath(Path::new("tests/oss/config.sample.toml")),
        )
        .await;

    match res {
        Ok(h) => println!("[success]\n{:#?}", h),
        Err(e) => println!("[error]\n{:#?}", e),
    }
}

#[tokio::test]
async fn get_object_test() {
    let client = get_oss_client();

    let res = client
        .get_object()
        .build()
        .receive_bytes("/test/sample.toml")
        .await;

    match res {
        Ok((data, header)) => {
            println!("[success] header: {:#?}", header);
            println!("[success] data: {}", String::from_utf8_lossy(&data));
        }
        Err(e) => println!("error: {}", e),
    }
}

#[tokio::test]
async fn get_object_by_download_test() {
    let client = get_oss_client();

    let res = client
        .get_object()
        .build()
        .download_to_file(
            "/test/sample.toml",
            Path::new("tests/oss/sample_download.toml"),
        )
        .await;

    match res {
        Ok(header) => println!("[success] header: {:#?}", header),
        Err(e) => println!("error: {}", e),
    }
}

#[tokio::test]
async fn get_object_by_stream_test() {
    let client = get_oss_client();

    let res = client
        .get_object()
        .build()
        .receive_bytes_stream("/test/sample.toml")
        .await;

    match res {
        Ok((mut stream, header)) => {
            println!("[success] header: {:#?}", header);
            let mut buf = String::new();
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(data) => buf.push_str(&String::from_utf8_lossy(&data)),
                    Err(e) => println!("stream error: {}", e),
                }
            }
            println!("[success] data: {}", buf);
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

// #[tokio::test]
// async fn append_object_test() {
//     let client = get_oss_client();
//
//     let append_header = AppendObjectHeader {
//         content_type: Some("text/plain"),
//         position: 13,
//         ..Default::default()
//     };
//     let res = client
//         .append_object(
//             "test/append_object.txt",
//             append_header,
//             None,
//             b"text123dfasdf".to_vec(),
//         )
//         .await;
//
//     match res {
//         Ok(next_pos) => {
//             println!("next_pos:{}", next_pos);
//         }
//         Err(e) => println!("error: {}", e),
//     }
// }

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
