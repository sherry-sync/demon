use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::path::PathBuf;

use regex::Regex;
use serde::{Serialize, Serializer};

pub fn ordered_map<S, K: Ord + Serialize, V: Serialize>(
    value: &HashMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

pub fn str_err_prefix<T: ToString + 'static>(prefix: &'static str) -> impl Fn(T) -> String {
    move |e| {
        let msg = format!("{}: {}", prefix, e.to_string());
        log::error!("{}", msg);
        msg
    }
}

pub const PATH_SEP: &str = "/";

pub fn normalize_path(p: &PathBuf) -> PathBuf {
    PathBuf::from(
        Regex::new(r"[\\/]+").unwrap()
            .replace_all(
                p.iter().collect::<Vec<&OsStr>>()
                    .join(OsStr::new(PATH_SEP))
                    .to_str().unwrap(),
                PATH_SEP).to_string()
    )
}
