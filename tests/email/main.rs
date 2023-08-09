mod test_config;

use u_ali_sdk::email::{
    utils::{get_uuid, now_iso8601},
    *,
};

#[test]
fn get_test_conf() {
    let s = test_config::AliConfig::get_conf();
    println!("{:?}", s);
}

#[test]
fn send_single_email() {
    let conf = test_config::AliConfig::get_conf();
    let client = EmailSdk::new(conf.access_key_id.clone(), conf.access_key_secret);
    let pub_p = pub_params::PubReqParams {
        format: Some("JSON".to_string()),
        version: "2015-11-23".to_string(),
        access_key_id: conf.access_key_id,
        signature_method: "HMAC-SHA1".to_string(),
        signature_version: "1.0".to_string(),
        timestamp: now_iso8601(),
        signature_nonce: get_uuid(),
        region_id: None,
    };
    let api_p = send_email::SingleSendEmailParams {
        account_name: conf.account_name,
        address_type: "1".to_string(),
        reply_to_address: "false".to_string(),
        subject: "test_subject".to_string(),
        to_address: conf.to_address,
        //TODO Action: SingleSendMail, 是必须的，到时后代码改一下
        action: Some("SingleSendMail".to_string()),
        click_trace: None,
        from_alias: None,
        html_body: Some("test 123".to_string()),
        tag_name: None,
        text_body: None,
        reply_address: None,
        reply_address_alias: None,
    };

    client.single_send_email(&pub_p, &api_p);
}
