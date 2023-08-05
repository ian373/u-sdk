mod test_config;

#[test]
fn get_test_conf() {
    let s = test_config::AliConfig::get_conf();
    println!("{:?}", s);
}
