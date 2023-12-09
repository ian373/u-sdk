mod test_config;
use u_ali_sdk::translate::*;

fn get_trans_client() -> TransClient {
    let conf = test_config::AliConfig::get_conf();
    TransClient::new(
        conf.access_key_id,
        conf.access_key_secret,
        "mt.aliyuncs.com".to_owned(),
    )
}

#[tokio::test]
async fn translate_test() {
    let client = get_trans_client();
    let query = types_rs::TranslateQuery {
        format_type: "text".to_owned(),
        source_language: "auto".to_owned(),
        target_language: "zh".to_owned(),
        source_text: "test first line.\ntest second line.".to_owned(),
        scene: "general".to_owned(),
        // scene: "description".to_owned(),
        context: None,
    };
    let res = client.translate(query).await;
    match res {
        Ok(s) => println!("res:\n{:#?}", s),
        Err(e) => println!("{:#?}", e),
    }
}

#[tokio::test]
async fn get_detect_language_test() {
    let client = get_trans_client();
    let res = client.get_detect_language("中文").await;
    match res {
        Ok(s) => println!("res:\n{}", s),
        Err(e) => println!("{:#?}", e),
    }
}
