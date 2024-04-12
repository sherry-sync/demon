use std::fs;
use std::io::Read;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn str_err_prefix<T: ToString + 'static>(prefix: &'static str) -> impl Fn(T) -> String {
    move |e| {
        let msg = format!("{}: {}", prefix, e.to_string());
        println!("{}", msg);
        log::info!("{}", msg);
        msg
    }
}

pub fn write_json_file<T, P: AsRef<Path>>(path: P, value: &T) -> Result<(), String>
    where
        T: ?Sized + serde::Serialize,
{
    fs::write(
        path,
        serde_json::to_string_pretty(value).map_err(str_err_prefix("Error JSON Encode"))?,
    ).map_err(str_err_prefix("Error File Write"))
}

pub fn get_file_string<P: AsRef<Path>>(path: P) -> Result<String, String> {
    let mut buf = String::new();
    fs::File::open(path).map_err(str_err_prefix("Error File Open"))?
        .read_to_string(&mut buf).map_err(str_err_prefix("Error Read String"))?;
    Ok(buf)
}

pub fn read_json_file<T, P: AsRef<Path>>(path: P) -> Result<T, String>
    where
        T: DeserializeOwned,
{
    serde_json::from_str(&get_file_string(path)?)
        .map_err(str_err_prefix("Error JSON Parse"))
}

pub fn initialize_json_file<T, P: AsRef<Path>>(path: P, default: T) -> Result<T, String>
    where
        T: DeserializeOwned + Serialize,
{
    match read_json_file(&path) {
        Ok(v) => Ok(v),
        Err(_) => {
            write_json_file(&path, &default)?;
            Ok(default)
        }
    }
}

pub fn initialize_json_file_with<T, P: AsRef<Path>, C>(path: P, default: &C) -> Result<T, String>
    where
        T: DeserializeOwned + Serialize,
        C: Fn() -> T
{
    match read_json_file(&path) {
        Ok(v) => Ok(v),
        Err(_) => {
            let value = default();
            write_json_file(&path, &value)?;
            Ok(value)
        }
    }
}