use itertools::join;
use itertools::Itertools;
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    convert::TryInto,
    fmt::Display,
    path::{Path, PathBuf},
    time::SystemTime,
};
use url::Url;

use crate::Tag;

#[derive(PartialEq, Eq, Hash)]
pub struct Bookmark {
    url: Url,
    tags: Vec<Tag>,
}

impl Bookmark {
    pub fn new<T: TryInto<Url>>(url: T, tags: Vec<Tag>) -> Result<Self, Error> {
        let url: Url = match url.try_into() {
            Ok(url) => url,
            Err(_) => return Err(Error::InvalidUrl),
        };
        let bm = Bookmark { url, tags };

        return Ok(bm);
    }

    pub fn from_file(path: &PathBuf) -> Option<Self> {
        if !path.exists() && !path.is_file() {
            return None;
        }

        let extension: Option<&str> = match path.extension() {
            Some(ext) => ext.to_str(),
            None => None,
        };

        if extension != Some("bkm") {
            return None;
        }

        let content: String = std::fs::read_to_string(path).ok()?;
        let content: Vec<&str> = content.lines().take(2).collect_vec();
        let url: &str = content.first()?;
        let tags: Vec<Tag> = Tag::new_vec(*content.last()?);

        Bookmark::new(url, tags).ok()
    }

    pub fn url(&self) -> Url {
        self.url.clone()
    }

    pub fn domain(&self) -> Option<&str> {
        self.url.domain()
    }

    pub fn tags(&self) -> &Vec<Tag> {
        &self.tags
    }

    pub fn contains(&self, tags: &Vec<Tag>) -> bool {
        tags.iter().any(|tag| self.tags().contains(tag))
    }

    pub fn rel_path(&self) -> PathBuf {
        let domain = self.domain().unwrap_or("").to_string();
        let mut hash = hash(&self.url.to_string());
        hash.push_str(".bkm");
        let path: String = [domain, hash].join("/");
        path.into()
    }

    pub fn merge(self, other: Bookmark) -> Bookmark {
        if self.url != other.url {
            self
        } else {
            let tags: Vec<Tag> = self
                .tags
                .iter()
                .chain(other.tags.iter())
                .map(|tag| tag.clone())
                .unique()
                .sorted()
                .collect();

            Bookmark { tags, ..self }
        }
    }
}

impl Display for Bookmark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n", &self.url)?;
        let tags: String = self.tags.iter().join(" ");
        write!(f, "{}\n", &tags)?;
        Ok(())
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
