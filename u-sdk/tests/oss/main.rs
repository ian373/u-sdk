use futures_util::StreamExt;
use oss::object::{ObjectToDelete, OssMetaExt, PutObjectBody};
use serde::Deserialize;
use std::path::Path;
use u_sdk::oss;

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

// #[tokio::test]
// async fn list_buckets_test() {
//     let client = get_oss_client();
//
//     let query = oss::service::ListBucketsQueryParams {
//         prefix: Some("test"),
//         ..Default::default()
//     };
//     let res = client.list_buckets(None, Some(query)).await;
//     match res {
//         Ok(s) => println!("res:\n{:#?}", s),
//         Err(e) => println!("{}", e),
//     }
// }

// #[tokio::test]
// async fn describe_regions_test() {
//     let client = get_oss_client();
//     let res = client.describe_regions(Some("oss-ap-northeast-1")).await;
//     match res {
//         Ok(s) => println!("res:\n{:#?}", s),
//         Err(e) => println!("{}", e),
//     }
// }

#[tokio::test]
async fn put_bucket_test() {
    let client = get_oss_client();
    let res = client
        .put_bucket()
        .bucket_name("example-to-del")
        .build()
        .send()
        .await;

    match res {
        Ok(header) => println!("[success] header: {:#?}", header),
        Err(e) => println!("[error] {}", e),
    }
}

#[tokio::test]
async fn list_objects_v2_test() {
    let client = get_oss_client();
    let res = client
        .list_objects_v2()
        .prefix("test/")
        .build()
        .send()
        .await;
    match res {
        Ok(s) => println!("[success] res:\n{:#?}", s),
        Err(e) => println!("[error] {}", e),
    }
}

#[tokio::test]
async fn get_bucket_info_test() {
    let client = get_oss_client();
    let res = client
        .get_bucket_info()
        .bucket("utab-app")
        .build()
        .send()
        .await;

    match res {
        Ok(info) => println!("[success] bucket info: {:#?}", info),
        Err(e) => println!("[error] {}", e),
    }
}

#[tokio::test]
async fn get_bucket_location_test() {
    let client = get_oss_client();
    let res = client
        .get_bucket_location()
        .bucket("utab-app")
        .build()
        .send()
        .await;

    match res {
        Ok(location) => println!("[success] bucket location: {}", location),
        Err(e) => println!("[error] {}", e),
    }
}

#[tokio::test]
async fn get_bucket_stat_test() {
    let client = get_oss_client();

    let res = client
        .get_bucket_stat()
        .bucket("utab-app")
        .build()
        .send()
        .await;

    match res {
        Ok(stat) => println!("[success] bucket stat: {:#?}", stat),
        Err(e) => println!("[error] {}", e),
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
            "test/t-sample.toml",
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
        .receive_bytes("test/t-sample.toml")
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
            "test/t-sample.toml",
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
        .receive_bytes_stream("test/t-sample.toml")
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

    let resp = client
        .copy_object()
        .x_oss_copy_source("/utab-app/custom-image/01919e65-75f0-7590-b8a8-b3f22f705db8")
        .unwrap()
        .build()
        .send(&client.bucket(), "test/copy_img.jpg")
        .await;

    match resp {
        Ok(res) => println!("[success] response: {:#?}", res),
        Err(e) => println!("[error] {}", e),
    }
}

#[tokio::test]
async fn append_object_test() {
    let client = get_oss_client();

    let res = client
        .append_object()
        .x_metas([("key1", "value1"), ("key2", "value2")])
        .content_type("text/plain")
        .build()
        .send("test/append_object_2.txt", 0, b"6666666".to_vec())
        .await;

    match res {
        Ok(header) => println!("[success] header: {:#?}", header),
        Err(e) => println!("[error] {}", e),
    }
}

#[tokio::test]
async fn delete_object_test() {
    let client = get_oss_client();
    let res = client.delete_object("test/IMG_20240726_155048.jpg").await;
    match res {
        Ok(h) => println!("[success] header: {:#?}", h),
        Err(e) => println!("[error] {}", e),
    }
}

#[tokio::test]
async fn delete_multiple_objects_test() {
    let client = get_oss_client();
    let objs = vec![
        ObjectToDelete {
            key: "test/copy_img.jpg",
            version_id: None,
        },
        ObjectToDelete {
            key: "test/sample.toml",
            version_id: None,
        },
    ];
    let res = client
        .delete_multiple_objects()
        .objects(objs)
        .quiet(false)
        .build()
        .send()
        .await;

    match res {
        Ok(result) => println!("[success] deleted objects: {:#?}", result),
        Err(e) => println!("[error] {}", e),
    }
}

#[tokio::test]
async fn head_object_test() {
    let client = get_oss_client();
    let res = client
        .head_object()
        .build()
        .send("test/test_88888.txt")
        .await;
    match res {
        Ok(header) => println!("[success] header: {:#?}", header),
        Err(e) => println!("[error] {}", e),
    }
}

#[tokio::test]
async fn get_object_meta_test() {
    let client = get_oss_client();
    let res = client.get_object_meta("test/test_88888.txt").await;
    match res {
        Ok(meta) => println!("[success] meta: {:#?}", meta),
        Err(e) => println!("[error] {}", e),
    }
}
