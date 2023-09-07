mod test_config;

// use u_ali_sdk::error::Error;
use u_ali_sdk::oss;

#[tokio::test]
async fn list_buckets_test() {
    let conf = test_config::AliConfig::get_conf();
    let client = oss::OSSClient::new(
        conf.access_key_id,
        conf.access_key_secret,
        conf.endpoint,
        conf.bucket_name,
    );
    let query = oss::service::ListBucketsQueryParams {
        prefix: None,
        marker: None,
        max_keys: None,
    };
    let res = client.list_buckets(None, query).await.unwrap();
    println!("res:\n{:?}", res);
}

#[tokio::test]
async fn describe_regions_test() {
    let conf = test_config::AliConfig::get_conf();
    let client = oss::OSSClient::new(
        conf.access_key_id,
        conf.access_key_secret,
        conf.endpoint,
        conf.bucket_name,
    );

    let res = client
        .describe_regions(Some("oss-cn-hangzhou"))
        .await
        .unwrap();
    println!("res:\n{:?}", res);
}

#[tokio::test]
async fn put_bucket_test() {
    use oss::bucket::{CreateBucketConfiguration, PutBucketHeader};

    let conf = test_config::AliConfig::get_conf();
    let client = oss::OSSClient::new(
        conf.access_key_id,
        conf.access_key_secret,
        conf.endpoint,
        conf.bucket_name,
    );

    let x_oss_header = PutBucketHeader {
        x_oss_acl: None,
        x_oss_resource_group_id: None,
    };
    let params = CreateBucketConfiguration {
        storage_class: None,
        data_redundancy_type: None,
    };

    client
        .put_bucket(
            x_oss_header,
            params,
            "oss-cn-hangzhou.aliyuncs.com",
            "example-oss",
        )
        .await
        .unwrap();
}
