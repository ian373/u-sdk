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

    let query = oss::service::ListBucketsQueryParams {
        prefix: None,
        marker: None,
        max_keys: None,
    };
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

    let x_oss_header = PutBucketHeader {
        x_oss_acl: None,
        x_oss_resource_group_id: None,
    };
    let params = CreateBucketConfiguration {
        storage_class: None,
        data_redundancy_type: None,
    };

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
