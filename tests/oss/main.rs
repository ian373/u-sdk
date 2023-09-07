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
