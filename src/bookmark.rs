use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    convert::TryInto,
    path::{Path, PathBuf},
    time::SystemTime,
};
use url::Url;

pub struct Bookmark {
    url: Url,
    timestamp: u64,
    tags: HashSet<String>,
}

impl Bookmark {
    pub fn new<T: TryInto<Url>>(url: T, tags: Option<HashSet<String>>) -> Result<Bookmark, Error> {
        let url: Url = match url.try_into() {
            Ok(url) => url,
            Err(_) => return Err(Error::InvalidUrl),
        };
        let tags: HashSet<String> = tags.unwrap_or_default();
        let timestamp: u64 = match Self::timestamp() {
            Some(ts) => ts,
            None => return Err(Error::NoTimestamp),
        };

        let bm = Bookmark {
            url,
            timestamp,
            tags,
        };

        return Ok(bm);
    }

    fn timestamp() -> Option<u64> {
        let now = SystemTime::now();
        let duration = now.duration_since(SystemTime::UNIX_EPOCH).ok();
        duration.map(|d| d.as_secs())
    }

    pub fn domain(&self) -> Option<&str> {
        self.url.domain()
    }

    pub fn rel_path(&self) -> PathBuf {
        let domain = self.domain().unwrap_or("").to_string();
        let hash = hash(&self.url.to_string());
        let path: String = [domain, hash].join("/");
        path.into()
    }
}

fn hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    format!("{:02x}", hasher.finalize())
}

#[derive(Debug)]
pub enum Error {
    InvalidUrl,
    NoDomain,
    NoTimestamp,
}
