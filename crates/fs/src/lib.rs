use sha2::{Digest, Sha256};
use std::{
    io,
    path::{Path, PathBuf},
};

pub mod error;

pub async fn file_exists_async(path: impl AsRef<Path>) -> eyre::Result<bool> {
    let path = path.as_ref();
    let is_file = tokio::fs::metadata(&path)
        .await
        .map(|m| m.is_file())
        .unwrap_or(false);
    Ok(is_file)
}

pub fn file_exists(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    path.is_file()
}

pub async fn dir_exists_async(path: impl AsRef<Path>) -> eyre::Result<bool> {
    let path = path.as_ref();
    let is_dir = tokio::fs::metadata(&path)
        .await
        .map(|m| m.is_dir())
        .unwrap_or(false);
    Ok(is_dir)
}

pub fn create_dir_if_needed(path: impl AsRef<Path>) -> Result<bool, io::Error> {
    let path = path.as_ref();
    if path.is_dir() {
        return Ok(false);
    }
    std::fs::create_dir(path)?;
    Ok(true)
}

pub fn dir_exists(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    path.is_dir()
}

pub fn gen_tmp_filename() -> String {
    let y0 = rand::random::<u128>();
    let y1 = rand::random::<u128>();
    format!("{y0:x}{y1:x}")
}

pub fn sha256(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    use std::fs::File;
    use std::io::{self};

    let mut file = File::open(path)?;

    let mut hasher = Sha256::new();
    let _n = io::copy(&mut file, &mut hasher)?;

    Ok(hasher.finalize().to_vec())
}

pub fn file_name_as_string(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string()
}

pub fn sha256_hex(path: impl AsRef<Path>) -> eyre::Result<String> {
    let h = sha256(path)?;
    Ok(hex::encode(h))
}

pub fn files_in_dir(path: impl AsRef<Path>) -> eyre::Result<Vec<PathBuf>> {
    let mut v: Vec<PathBuf> = vec![];
    if !dir_exists(&path) {
        return Ok(v);
    }

    let read_dir = match std::fs::read_dir(&path) {
        Ok(rd) => rd,
        Err(_) => return Ok(v),
    };

    for r_dir_entry in read_dir {
        let entry = match r_dir_entry {
            Ok(de) => de,
            Err(_) => continue,
        };
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        if !file_type.is_file() {
            continue;
        }

        let abs_filename = std::fs::canonicalize(entry.path())?;
        v.push(abs_filename);
    }

    Ok(v)
}

pub fn absolute_path(path: impl AsRef<Path>) -> Result<PathBuf, io::Error> {
    let path = path.as_ref();

    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        let cd = std::env::current_dir()?;
        Ok(cd.join(path))
    }
}

pub fn get_workspace_dir() -> Result<PathBuf, error::FsError> {
    use error::FsError;

    let dir = env!("CARGO_MANIFEST_DIR");
    let pb = PathBuf::from(dir).join("../..");
    if pb.is_dir() {
        Ok(std::fs::canonicalize(&pb)?)
    } else {
        Err(FsError::Unexpected(format!("env `CARGO_MANIFEST_DIR` is undefined or has empty value, and the pre-defined data dir `{}` not found", String::from(pb.to_str().unwrap_or("")))))
    }
}

pub fn get_test_files_dir() -> Result<PathBuf, error::FsError> {
    Ok(get_workspace_dir()?.join("tests-data/files"))
}

pub fn get_test_db_dir() -> Result<PathBuf, error::FsError> {
    Ok(get_workspace_dir()?.join("tests-data/db"))
}

#[cfg(test)]
mod test {
    use crate::{files_in_dir, get_test_files_dir, sha256, sha256_hex};

    #[test]
    fn test_sha256() {
        let dir = get_test_files_dir().unwrap();

        let mut v = files_in_dir(&dir).unwrap();
        v.sort();

        let expected_results = [
            "edeaaff3f1774ad2888673770c6d64097e391bc362d7d6fb34982ddf0efd18cb",
            "1c27ae443e93ef623d8670b611ae1d7f7d71c7f103258ff8ce0c90fab557dfd8",
            "c6c120919b642caa47945b43e69c5aaeb844d552a2d64f4292b300051d6be614",
            "0042ef9db7a139333989d8fa47a3e0228544be49e4a8438d33dd648c31df154f",
            "047ba34157119793874a19ecc95af8507e5536a334a63137cb54ffe8cb33cab3",
            "624c70a025bc8977861c4f48c893332910c4d61a3bfccd4a2c435ffd35b16751",
        ];
        assert_eq!(v.len(), expected_results.len());
        for i in 0..v.len() {
            let hash = sha256(&v[i]).unwrap();
            assert_eq!(hash.len(), 32);
            let h = sha256_hex(&v[i]).unwrap();
            assert_eq!(h, expected_results[i]);
        }
    }
}
