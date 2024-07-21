use crate::error::Error;

pub fn get_local_file(file_path: &str) -> Result<(String, Vec<u8>), Error> {
    let p = std::path::Path::new(file_path);
    // 调用is_file，将检查本地磁盘是否存在该文件
    if !p.is_file() {
        return Err(Error::AnyError(format!(
            "The file doesn't exist or not have permission to access, path: {}",
            file_path
        )));
    }

    let bytes = std::fs::read(file_path)
        .map_err(|_| Error::AnyError(format!("Faild to read the file with path: {}", file_path)))?;

    // pub object API限制文件大小不超过5G
    if bytes.len() > 5 * 1024 * 1024 {
        return Err(Error::AnyError(
            "Can't upload file which size is > 5G".to_owned(),
        ));
    }

    let mut file_name = String::from(p.file_stem().unwrap().to_str().unwrap());
    // 文件名后缀，同一转化为小写
    if let Some(ex) = p.extension() {
        file_name.push('.');
        file_name.push_str(ex.to_str().unwrap().to_lowercase().as_str())
    }

    Ok((file_name, bytes))
}

#[test]
fn get_local_file_test() {
    let res = get_local_file(r"C:\es\test_no_ex");
    match res {
        Ok((s, _)) => println!("name: {}", s),
        Err(e) => println!("{}", e),
    }
}

pub fn get_dest_path(path: &str, local_file_name: &str) -> Result<String, Error> {
    let mut dest_path = std::path::PathBuf::from(path);
    if !dest_path.has_root() {
        return Err(Error::AnyError("Please input a absoulute path".to_owned()));
    }

    if path.ends_with('/') {
        dest_path.push(local_file_name);
    }

    Ok(dest_path.to_str().unwrap().to_owned())
}

#[test]
fn get_dest_path_test() {
    let res = get_dest_path(r"/", "123.txt");
    match res {
        Ok(s) => println!("res:{}", s),
        Err(e) => println!("error:{}", e),
    }
}
