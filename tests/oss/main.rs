mod test_config;

use oss::object::types_rs::*;
use std::collections::HashMap;
use u_ali_sdk::oss;

fn get_oss_client() -> oss::OSSClient {
    let conf = test_config::AliConfig::get_conf();
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

    let res = client
        .get_object(c_header, "/test/test_fi.txt", r"C:\Users\666.txt")
        .await;

    match res {
        Ok(r) => {
            if let Some(map) = r {
                println!("ok, and headers is: {:#?}", map);
            } else {
                println!("ok!");
            }
        }
        Err(_e) => println!("error: {:?}", _e),
    }
}

#[tokio::test]
async fn copy_object_test() {
    let client = get_oss_client();

    let x_header = CopyObjectXHeader {
        x_oss_forbid_overwrite: Some("true"),
        ..Default::default()
    };

    let res = client
        .copy_object(
            "/uua-private-01/13468799.TXT",
            Some("example-oss-todel"),
            None,
            "/1122/123.txt",
            x_header,
        )
        .await;

    match res {
        Ok(r) => {
            println!("res:{:#?}", r);
        }
        Err(e) => println!("error: {:?}", e),
    }
}

#[tokio::test]
async fn append_object_test() {
    unimplemented!("not implemented");
}

#[tokio::test]
async fn delete_object_test() {
    let client = get_oss_client();

    let res = client.delete_object("/123.txt").await;

    match res {
        Ok(_) => {
            println!("OK!");
        }
        Err(e) => println!("error: {:?}", e),
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

    let res = client.delete_multiple_objects(None, objs, false).await;

    match res {
        Ok(s) => {
            println!("ok_res:{:#?}", s);
        }
        Err(e) => println!("error: {:?}", e),
    }
}

#[tokio::test]
async fn head_object_test() {
    let client = get_oss_client();

    let req_header = HeadObjectHeader::default();

    let res = client.head_object("/test_path/123.txt", req_header).await;

    match res {
        Ok(s) => {
            println!("ok_res:{:#?}", s);
        }
        Err(e) => println!("error: {:?}", e),
    }
}

#[tokio::test]
async fn get_object_meta_test() {
    let client = get_oss_client();

    let res = client.get_object_meta("/test_path/123.txt").await;

    match res {
        Ok(s) => {
            println!("ok_res:{:#?}", s);
        }
        Err(e) => println!("error: {:?}", e),
    }
}
