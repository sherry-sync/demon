use std::future::Future;
use std::path::{Path, PathBuf};

use futures::{Stream, StreamExt};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::bytes::Bytes;

use crate::helpers::str_err_prefix;

pub async fn write_json_file<T, P: AsRef<Path>>(path: P, value: &T) -> Result<(), String>
    where
        T: ?Sized + serde::Serialize,
{
    fs::write(
        path,
        serde_json::to_string_pretty(value).map_err(str_err_prefix("Error JSON Encode"))?,
    ).await.map_err(str_err_prefix("Error File Write"))
}

pub async fn get_file_string<P: AsRef<Path>>(path: P) -> Result<String, String> {
    let mut buf = String::new();
    fs::File::open(path).await
        .map_err(str_err_prefix("Error File Open"))?
        .read_to_string(&mut buf).await
        .map_err(str_err_prefix("Error Read String"))?;
    Ok(buf)
}

pub async fn read_json_file<T, P: AsRef<Path>>(path: P) -> Result<T, String>
    where
        T: DeserializeOwned,
{
    serde_json::from_str(&get_file_string(path).await?)
        .map_err(str_err_prefix("Error JSON Parse"))
}

pub async fn initialize_json_file<T, P: AsRef<Path>>(path: P, default: T) -> Result<T, String>
    where
        T: DeserializeOwned + Serialize,
{
    match read_json_file(&path).await {
        Ok(v) => Ok(v),
        Err(_) => {
            write_json_file(&path, &default).await?;
            Ok(default)
        }
    }
}

pub async fn initialize_json_file_with<T, P: AsRef<Path>, C, Fut>(path: P, default: &C) -> Result<T, String>
    where
        T: DeserializeOwned + Serialize,
        C: Fn() -> Fut,
        Fut: Future<Output=T>,
{
    match read_json_file(&path).await {
        Ok(v) => Ok(v),
        Err(_) => {
            let value = default().await;
            write_json_file(&path, &value).await?;
            Ok(value)
        }
    }
}

async fn create_file(path: &PathBuf) -> Result<fs::File, String> {
    match fs::create_dir_all(path.parent().unwrap()).await {
        Err(e) => {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(e).map_err(str_err_prefix("Error Dir Create"));
            }
        }
        _ => (),
    };
    fs::File::create(path).await.map_err(str_err_prefix("Error File Create"))
}

pub async fn write_file_from_stream(path: &PathBuf, mut stream: impl Stream<Item=Result<Bytes, reqwest::Error>> + Unpin) -> Result<(), String> {
    let mut file = create_file(path).await?;
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(str_err_prefix("Invalid chunk"))?;
        file.write_all(&chunk).await.map_err(str_err_prefix("Error Write"))?;
    }
    Ok(())
}

pub async fn write_files_from_stream(paths: &Vec<PathBuf>, mut stream: impl Stream<Item=Result<Bytes, reqwest::Error>> + Unpin) -> Result<(), String> {
    let mut files = futures::future::join_all(paths.iter().map(|p| create_file(&p))).await.into_iter().filter_map(|v| v.ok()).collect::<Vec<fs::File>>();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(str_err_prefix("Invalid chunk"))?;
        for file in files.iter_mut() {
            file.write_all(&chunk).await.map_err(str_err_prefix("Error Write"))?
        }
    }

    Ok(())
}

pub async fn delete_path(path: &PathBuf) -> Result<(), String> {
    if path.is_dir() {
        fs::remove_dir_all(&path).await.map_err(str_err_prefix(format!("Error Dir Remove at {}", &path.to_str().unwrap())))?;
    } else {
        fs::remove_file(path).await.map_err(str_err_prefix(format!("Error File Remove at {}", &path.to_str().unwrap())))?;
    }
    Ok(())
}

pub async fn rename_path(old: &PathBuf, new: &PathBuf) -> Result<(), String> {
    fs::rename(old, new).await.map_err(str_err_prefix("Error File/Folder Rename"))?;
    Ok(())
}
