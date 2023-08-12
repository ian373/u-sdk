mod test_config;
use u_ali_sdk::blocking::email as email_blocking;
use u_ali_sdk::email;

// 获取本地配置信息测试
#[test]
fn get_test_conf() {
    let s = test_config::AliConfig::get_conf();
    println!("{:?}", s);
}

#[test]
fn blocking_single_send_email() {
    let conf = test_config::AliConfig::get_conf();
    let client = email_blocking::EmailSdk::new(conf.access_key_id, conf.access_key_secret, None);
    let params = email::send_email::SingleSendEmailParams {
        account_name: &conf.account_name,
        address_type: "1",
        reply_to_address: "false",
        subject: "test_subject邮件发送测试",
        to_address: &conf.to_address,
        click_trace: None,
        from_alias: None,
        html_body: Some("这是一个测试（blocking）"),
        tag_name: None,
        text_body: None,
        reply_address: None,
        reply_address_alias: None,
    };

    client.single_send_email(&params);
}

#[tokio::test]
async fn single_send_email() {
    let conf = test_config::AliConfig::get_conf();
    let client = email::EmailSdk::new(conf.access_key_id, conf.access_key_secret, None);
    let params = email::send_email::SingleSendEmailParams {
        account_name: &conf.account_name,
        address_type: "1",
        reply_to_address: "false",
        subject: "test_subject",
        to_address: &conf.to_address,
        click_trace: None,
        from_alias: None,
        html_body: Some("这是一个异步的测试 123"),
        tag_name: None,
        text_body: None,
        reply_address: None,
        reply_address_alias: None,
    };

    client.single_send_email(&params).await;
}
