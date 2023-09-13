use crate::error::Error;

pub fn get_local_file(file_path: &str) -> Result<(String, Vec<u8>), Error> {
    let p = std::path::Path::new(file_path);
    let file_path = p.to_str().ok_or(Error::CommonError(
        "Please input a valid file path!".to_owned(),
    ))?;
    if !p.is_file() {
        return Err(Error::CommonError("Please input a file path!".to_owned()));
    }

    let bytes = std::fs::read(file_path).map_err(|_| {
        Error::CommonError(format!("Faild to read the file with path: {}", file_path))
    })?;

    // pub object API限制文件大小不超过5G
    if bytes.len() > 5 * 1024 * 1024 {
        return Err(Error::CommonError(
            "Can't upload file which size is > 5G".to_owned(),
        ));
    }

    Ok((
        p.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
            .to_lowercase(),
        bytes,
    ))
}
