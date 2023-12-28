use time::format_description::well_known::iso8601::{
    Config, EncodedConfig, Iso8601, TimePrecision,
};

pub fn get_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn now_gmt() -> String {
    use time::format_description::well_known::Rfc2822;
    time::OffsetDateTime::now_utc()
        .format(&Rfc2822)
        .unwrap()
        .replace("+0000", "GMT")
}

pub fn now_iso8601() -> String {
    const ENCODED_CONFIG: EncodedConfig = Config::DEFAULT
        .set_time_precision(TimePrecision::Second {
            decimal_digits: None,
        })
        .encode();

    time::OffsetDateTime::now_utc()
        .format(&Iso8601::<ENCODED_CONFIG>)
        .unwrap()
}
