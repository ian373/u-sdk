mod test_config;

use u_ali_sdk::email::*;

#[test]
fn get_test_conf() {
    let s = test_config::AliConfig::get_conf();
    println!("{:?}", s);
}

#[test]
fn send_single_email() {
    let conf = test_config::AliConfig::get_conf();
    let client = EmailSdk::new(conf.access_key_id, conf.access_key_secret, None);
    let params = send_email::SingleSendEmailParams {
        account_name: &conf.account_name,
        address_type: "1",
        reply_to_address: "false",
        subject: "test_subject邮件发送测试",
        to_address: &conf.to_address,
        click_trace: None,
        from_alias: None,
        html_body: Some("这是一个测试+ 123 `123`"),
        tag_name: None,
        text_body: None,
        reply_address: None,
        reply_address_alias: None,
    };

    client.single_send_email(&params);
}

#[tokio::test]
async fn async_send_email() {
    let conf = test_config::AliConfig::get_conf();
    let client =
        async_send_email::EmailSdkAsync::new(conf.access_key_id, conf.access_key_secret, None);
    let params = async_send_email::SingleSendEmailParams {
        account_name: conf.account_name,
        address_type: "1".to_string(),
        reply_to_address: "false".to_string(),
        subject: "test_subject".to_string(),
        to_address: conf.to_address,
        click_trace: None,
        from_alias: None,
        html_body: Some("这是一个异步的测试 123".to_string()),
        tag_name: None,
        text_body: None,
        reply_address: None,
        reply_address_alias: None,
    };

    client.single_send_email_async(&params).await;
}
