use crate::translate::Error;

pub(crate) async fn parse_json_response<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, Error> {
    let status = resp.status();
    if !status.is_success() {
        return Err(Error::RequestAPIFailed {
            code: status.to_string(),
            message: resp.text().await?,
        });
    }

    let data = resp.json().await?;
    Ok(data)
}
