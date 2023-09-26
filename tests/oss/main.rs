mod test_config;

use u_ali_sdk::error::Error;
use u_ali_sdk::oss;

fn get_oss_client() -> oss::OSSClient {
    let conf = test_config::AliConfig::get_conf();
    oss::OSSClient::new(
        conf.access_key_id,
        conf.access_key_secret,
        conf.endpoint,
        conf.bucket_name,
    )
}

#[tokio::test]
async fn list_buckets_test() {
    let client = get_oss_client();

    let query = oss::service::ListBucketsQueryParams::default();
    let res = client.list_buckets(None, query).await;
    match res {
        Ok(s) => println!("res:\n{:?}", s),
        Err(e) => println!("{:?}", e),
    }
}

#[tokio::test]
async fn describe_regions_test() {
    let client = get_oss_client();

    let res = client.describe_regions(Some("oss-cn-hangzhou")).await;

    match res {
        Ok(s) => println!("res:\n{:?}", s),
        Err(e) => println!("{:?}", e),
    }
}

#[tokio::test]
async fn put_bucket_test() {
    use oss::bucket::{CreateBucketConfiguration, PutBucketHeader};

    let client = get_oss_client();

    let x_oss_header = PutBucketHeader::default();
    let params = CreateBucketConfiguration::default();

    let res = client
        .put_bucket(
            x_oss_header,
            params,
            "oss-cn-hangzhou.aliyuncs.com",
            "example-oss-todel",
        )
        .await;

    match res {
        Ok(_) => println!("success!"),
        Err(e) => println!("{:?}", e),
    }
}

#[tokio::test]
async fn list_objects_v2_test() {
    use oss::bucket::ListObjectsV2Query;

    let client = get_oss_client();

    let params = ListObjectsV2Query {
        delimiter: None,
        start_after: Some("t"),
        continuation_token: None,
        max_keys: Some("3"),
        prefix: Some("test/"),
        encoding_type: None,
        fetch_owner: Some("true"),
    };

    let res = client.list_objects_v2(params).await;

    match res {
        Ok(s) => println!("res:\n {:?}", s),
        Err(e) => match e {
            Error::StatusCodeNot200Resp(resp) => println!("text: {}", resp.text().await.unwrap()),
            _ => println!("error: {}", e),
        },
    }
}

#[tokio::test]
async fn get_bucket_info_test() {
    let client = get_oss_client();

    let res = client.get_bucket_info(Some("example-oss-todel")).await;

    match res {
        Ok(s) => println!("res:\n {:?}", s),
        Err(e) => match e {
            Error::StatusCodeNot200Resp(resp) => println!("text: {}", resp.text().await.unwrap()),
            _ => println!("{}", e),
        },
    }
}

#[tokio::test]
async fn get_bucket_location_test() {
    let client = get_oss_client();

    let res = client.get_bucket_location(Some("example-oss-todel")).await;

    match res {
        Ok(s) => println!("res:\n {}", s),
        Err(e) => match e {
            Error::StatusCodeNot200Resp(resp) => println!("text: {}", resp.text().await.unwrap()),
            _ => println!("{}", e),
        },
    }
}

#[tokio::test]
async fn get_bucket_stat_test() {
    let client = get_oss_client();

    let res = client.get_bucket_stat(Some("example-oss-todel")).await;

    match res {
        Ok(s) => println!("res:\n {:?}", s),
        Err(e) => match e {
            Error::StatusCodeNot200Resp(resp) => println!("text: {}", resp.text().await.unwrap()),
            _ => println!("{}", e),
        },
    }
}

#[tokio::test]
async fn put_object_test() {
    use oss::object::basic::*;
    use std::collections::HashMap;

    let client = get_oss_client();

    let c_header = CHeader::default();
    let x_header = XHeader {
        x_oss_forbid_overwrite: Some("true"),
        ..Default::default()
    };
    let mut x_meta_map = HashMap::new();
    x_meta_map.insert("test-1", "test-v-1");

    let res = client
        .put_object(
            c_header,
            x_header,
            x_meta_map.into(),
            r"C:\ex\a\123.txt",
            "/test_file/",
            None,
        )
        .await;

    match res {
        Ok(s) => println!("res:\n {:?}", s),
        Err(e) => match e {
            Error::StatusCodeNot200Resp(resp) => println!("text: {}", resp.text().await.unwrap()),
            _ => println!("{}", e),
        },
    }
}

#[tokio::test]
async fn get_object_test() {
    use oss::object::basic::*;

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
        Err(e) => match e {
            Error::StatusCodeNot200Resp(resp) => println!("text: {}", resp.text().await.unwrap()),
            _ => println!("{}", e),
        },
    }
}

#[tokio::test]
async fn copy_object_test() {
    use oss::object::basic::*;

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
        Err(e) => match e {
            Error::StatusCodeNot200Resp(resp) => println!("text: {}", resp.text().await.unwrap()),
            _ => println!("{}", e),
        },
    }
}

#[tokio::test]
async fn append_object_test() {
    use oss::object::basic::*;

    let client = get_oss_client();

    let c_header = AppendObjectCHeader::default();
    let x_header = AppendObjectXHeader::default();

    let res = client
        .append_object(
            b"test123".to_vec(),
            "/test_path/123.txt",
            None,
            14,
            c_header,
            x_header,
            None,
        )
        .await;

    match res {
        Ok(s) => {
            println!("OK: {:#?}", s);
        }
        Err(e) => match e {
            Error::StatusCodeNot200Resp(resp) => println!("text: {}", resp.text().await.unwrap()),
            _ => println!("{}", e),
        },
    }
}
