#![cfg(feature = "oss")]

use oss::object::{ObjectToDelete, OssMetaExt, PutObjectBody};
use serde::Deserialize;
use std::path::Path;
use std::time::Duration;
use time::OffsetDateTime;
use tokio_stream::StreamExt;
use u_sdk::oss;
use u_sdk::oss::object::{CallBackBody, CallbackBodyType, OssCallBack};

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
        toml::from_str(&file_str).unwrap()
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
#[ignore]
async fn list_buckets_test() {
    let client = get_oss_client();
    let res = client.list_buckets().build().send().await;
    match res {
        Ok(buckets) => println!("[success] buckets: {:#?}", buckets),
        Err(e) => println!("[error] {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn describe_regions_test() {
    let client = get_oss_client();
    let res = client
        .describe_regions()
        .region("oss-cn-hangzhou")
        .build()
        .send()
        .await;
    match res {
        Ok(region_info_list) => println!("[success] region info list: {:#?}", region_info_list),
        Err(e) => println!("[error] {}", e),
    }
}

#[tokio::test]
#[ignore]
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
#[ignore]
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
#[ignore]
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
#[ignore]
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
#[ignore]
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
#[ignore]
async fn put_object_test() {
    let client = get_oss_client();

    let callback_body = CallBackBody::builder()
        .bucket(true)
        .object(true)
        .vars([("uid", "uid", "1234"), ("order", "order_id", "1234")])
        .build();
    let callback = OssCallBack::builder()
        .callback_url(["https://webhook.site/ff5086fe-20db-43d3-803a-b6955c139f88"])
        .callback_body(callback_body)
        .callback_sni(false)
        .callback_body_type(CallbackBodyType::Json)
        .build();

    let res = client
        .put_object()
        .content_type("text/plain")
        .cache_control("max-age=6666")
        .x_meta("key", "value")
        .x_metas([("key3", "value3"), ("key4", "value4")])
        .callback(callback)
        .build()
        .send(
            "test-ffb/t-sample.toml",
            PutObjectBody::FilePath(Path::new("tests/oss/config.sample.toml")),
        )
        .await;

    match res {
        Ok(h) => println!("[success]\n{:#?}", h),
        Err(e) => println!("[error]\n{:#?}", e),
    }
}

#[test]
#[ignore]
fn put_object_presigned_url_test() {
    let client = get_oss_client();
    let callback_body = CallBackBody::builder()
        .bucket(true)
        .object(true)
        .vars([("uid", "uid", "1234"), ("order", "order_id", "1234")])
        .build();
    let callback = OssCallBack::builder()
        .callback_url(["https://example.com/webhook/oss/callback"])
        .callback_body(callback_body)
        .callback_sni(false)
        .callback_body_type(CallbackBodyType::Json)
        .build();
    let res = client
        .put_object()
        .content_type("text/plain")
        // .cache_control("max-age=6666")
        .x_meta("key", "value")
        .x_metas([("key3", "value3"), ("key4", "value4")])
        .callback(callback)
        .build()
        .generate_presigned_url("test/k-sample.toml", 300);
    /*
    在客户端使用生成的Presigned URL进行 PUT 请求时，
    请求的header中需要包含构建PutObject设置的那些header以及对应的值，如上面的例子中需要指定
    Content-Type: text/plain
    Cache-Control: max-age=6666
    x-oss-meta-key: value
    x-oss-meta-key3: value3
    x-oss-meta-key4: value4
    然后body中放置Binary[单个文件]

    客户端请求的时候header只能包含在构建PutObject中设置的那些header，不能包含其他header，否则会导致签名验证失败。
     */
    println!("res: {:#?}", res);
}

#[test]
#[ignore]
fn generate_post_object_policy_test() {
    let now = OffsetDateTime::now_utc();
    let client = get_oss_client();

    let callback_body = CallBackBody::builder()
        .bucket(true)
        .object(true)
        .vars([("uid", "uid", "1234"), ("order", "order_id", "1234")])
        .build();
    let callback = OssCallBack::builder()
        .callback_url(["https://example.com/webhook/oss/callback"])
        .callback_body(callback_body)
        .callback_sni(false)
        .callback_body_type(CallbackBodyType::Json)
        .build();

    let res = client
        .post_object()
        .content_length_range((1, 1024 * 1024))
        .key(("eq".to_owned(), "test2/t-sample7777.txt".to_owned()))
        .x_oss_content_type("text/plain".to_owned())
        .x_meta("kk1", ("eq", "vv1"))
        .x_metas([("kk2", ("eq", "vv2")), ("kk3", ("starts-with", "vv3"))])
        .callback(callback)
        .build()
        .generate_policy(now + Duration::from_secs(900));
    println!("res: {:#?}", res);

    /*
    前端使用post，form-data类型上传。url为https://bucket.endpoint/，如https://my-bucket.oss-cn-hangzhou.aliyuncs.com
    对于构建PostObject设置的各个字段，前端上传时必须存在，且值必须满足condition中设置的条件。
    form-data的最后一个字段一定是`file`，类型为file类型，且只允许上传单个文件
    对于PostObject中未涉及的其它api字段，前端上传时可以自己额外添加，不会影响签名验证，而且这些字段也会对请求起作用

    对于上面的例子，前端form-data中必须包含以下字段：
    // post object签名要求的字段
    x-oss-signature-version: OSS4-HMAC-SHA256
    x-oss-credential: sdk返回的credential字符串
    x-oss-date： sdk返回的date字符串
    x-oss-signature： sdk返回的signature字符串
    policy： sdk返回的policy字符串
    key: 如果构建PostObject时指定了key的condition则需要满足其要求，否则可以为任意值。
    (需要注意的是：如果构建PostObject时未指定key的condition，则前端上传时可以为任意值，也就意味着前端可以在有效期内上传任意个object)
    key: test2/t-sample7777.txt
    // post object api字段
    x-oss-content-type: text/plain
    x-oss-meta-kk1: vv1
    x-oss-meta-kk2: vv2
    x-oss-meta-kk3: vv3
    // callback相关字段
    callback: base64编码后的callback字符串
    [callback-var] sdk返回的HashMap的内容每一个k,v对都作为一个字段加入form-data，k为字段名，v为字段值
    file: 除了file字段需要在最后，其它字段的顺序不限
     */
}

#[tokio::test]
#[ignore]
async fn get_object_by_bytes_test() {
    let client = get_oss_client();

    let res = client
        .get_object()
        .range("bytes=0-99")
        .response_content_language("en-US")
        .build()
        .receive_bytes("test/t-sample.toml")
        .await;

    match res {
        Ok((data, response_header, _)) => {
            println!("[success] header: {:#?}", response_header);
            println!("[success] data: {}", String::from_utf8_lossy(&data));
        }
        Err(e) => println!("error: {}", e),
    }
}

#[tokio::test]
#[ignore]
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
        Ok((header, _)) => println!("[success] header: {:#?}", header),
        Err(e) => println!("error: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn get_object_by_stream_test() {
    let client = get_oss_client();

    let res = client
        .get_object()
        .build()
        .receive_bytes_stream("test/t-sample.toml")
        .await;

    match res {
        Ok((mut stream, response_header, _)) => {
            println!("[success] header: {:#?}", response_header);
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

#[test]
#[ignore]
fn get_object_presigned_url_test() {
    let client = get_oss_client();
    let res = client
        .get_object()
        .range("bytes=0-99")
        .response_content_language("en-US")
        .build()
        .generate_presigned_url("test/t-sample.toml", 300);
    println!("res: {:#?}", res);
    /*
    使用sdk生成的Presigned URL进行 GET 请求时，
    请求的header中需要包含构建GetObject设置的那些header以及对应的值，否则会导致签名验证失败。
    对于其它未包含在构建GetObject中的header，如果前端请求的时候自己添加了这些header，是不会影响签名验证的，但是对请求会起作用。
     */
}

#[tokio::test]
#[ignore]
async fn copy_object_test() {
    let client = get_oss_client();

    let resp = client
        .copy_object()
        .x_oss_copy_source("/utab-app/custom-image/01919e65-75f0-7590-b8a8-b3f22f705db8")
        .unwrap()
        .build()
        .send(client.bucket(), "test/copy_img.jpg")
        .await;

    match resp {
        Ok(res) => println!("[success] response: {:#?}", res),
        Err(e) => println!("[error] {}", e),
    }
}

#[tokio::test]
#[ignore]
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
#[ignore]
async fn delete_object_test() {
    let client = get_oss_client();
    let res = client.delete_object("test/IMG_20240726_155048.jpg").await;
    match res {
        Ok(h) => println!("[success] header: {:#?}", h),
        Err(e) => println!("[error] {}", e),
    }
}

#[tokio::test]
#[ignore]
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
#[ignore]
async fn head_object_test() {
    let client = get_oss_client();
    let res = client
        .head_object()
        .build()
        .send("test/test_88888.txt")
        .await;
    match res {
        Ok((header, _)) => println!("[success] header: {:#?}", header),
        Err(e) => println!("[error] {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn get_object_meta_test() {
    let client = get_oss_client();
    let res = client.get_object_meta("test/test_88888.txt").await;
    match res {
        Ok(meta) => println!("[success] meta: {:#?}", meta),
        Err(e) => println!("[error] {}", e),
    }
}
