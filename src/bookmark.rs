use std::{collections::HashSet, time::SystemTime};
use url::Url;

struct Bookmark {
    url: Url,
    timestamp: u64,
    tags: HashSet<String>,
}

impl Bookmark {
    pub fn new(url: &str, tags: Option<HashSet<String>>) -> Result<Bookmark, Error> {
        let url: Url = match Url::parse(url) {
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
}

#[derive(Debug)]
enum Error {
    InvalidUrl,
    NoDomain,
    NoTimestamp,
}
