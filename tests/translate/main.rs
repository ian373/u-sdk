mod test_config;
use u_ali_sdk::translate::*;

fn get_trans_client() -> TransClient {
    let conf = test_config::AliConfig::get_conf();
    TransClient::new(
        conf.access_key_id,
        conf.access_key_secret,
        "mt.aliyuncs.com".to_owned(),
        50,
        5000,
    )
}

#[tokio::test]
async fn translate_general_test() {
    let client = get_trans_client();
    let query = trans::GeneralTranslateQuery {
        format_type: "text".to_owned(),
        source_language: "en".to_owned(),
        target_language: "zh".to_owned(),
        source_text: "this is a test.".to_owned(),
        scene: "general".to_owned(),
        context: None,
    };
    client.general_translate(query).await;
}
