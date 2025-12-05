use super::Error;

pub(crate) async fn parse_json_response<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, Error> {
    let status = resp.status();

    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::API { code: status, body });
    }

    let json = resp.json::<T>().await?;
    Ok(json)
}
