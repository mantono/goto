use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashSet, convert::TryInto, fmt::Display, path::PathBuf};
use std::{hash::Hash, path::Path};
use url::Url;

use crate::tag::Tag;

#[derive(Eq, Serialize, Deserialize, Clone)]
pub struct Bookmark {
    url: Url,
    title: Option<String>,
    tags: HashSet<Tag>,
}

impl Bookmark {
    pub fn new<T: TryInto<Url>>(
        url: T,
        title: Option<String>,
        tags: HashSet<Tag>,
    ) -> Result<Self, Error> {
        let url: Url = match url.try_into() {
            Ok(url) => url,
            Err(_) => return Err(Error::InvalidUrl),
        };
        let bm = Bookmark { url, title, tags };

        return Ok(bm);
    }

    pub fn from_file(path: &Path) -> Option<Self> {
        if !path.exists() && !path.is_file() {
            return None;
        }

        let extension: Option<&str> = match path.extension() {
            Some(ext) => ext.to_str(),
            None => None,
        };

        if extension != Some("json") {
            return None;
        }

        let bytes = std::fs::read(path).ok()?;
        Self::from_bytes(&bytes)
    }

    pub fn from_str(input: &str) -> Option<Self> {
        serde_json::from_str(input).ok()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()
    }

    fn id(&self) -> String {
        hash(&self.url.to_string())
    }

    pub fn url(&self) -> Url {
        self.url.clone()
    }

    pub fn domain(&self) -> Option<&str> {
        self.url.domain()
    }

    fn root_domain(&self) -> Option<&str> {
        let parts: Vec<&str> = self.domain()?.split(".").collect();
        parts.iter().nth_back(1).map(|r| *r)
    }

    pub fn title(&self) -> Option<String> {
        self.title.clone()
    }

    pub fn tags(&self) -> &HashSet<Tag> {
        &self.tags
    }

    pub fn terms(&self) -> HashSet<Tag> {
        let mut terms: HashSet<Tag> = self.tags.clone();
        if let Some(domain) = self.root_domain() {
            Tag::new(domain).ok().and_then(|d| Some(terms.insert(d)));
        };
        terms
    }

    pub fn rel_path(&self) -> PathBuf {
        let domain = self.domain().unwrap_or("").to_string();
        let mut hash = self.id();
        hash.push_str(".json");
        let path: String = [domain, hash].join("/");
        path.into()
    }

    pub fn merge(self, other: Bookmark) -> Bookmark {
        if self.url != other.url {
            self
        } else {
            let tags: HashSet<Tag> = self
                .tags
                .iter()
                .chain(other.tags.iter())
                .map(|tag| tag.clone())
                .collect();

            Bookmark { tags, ..self }
        }
    }
}

impl Display for Bookmark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tags: String = self.tags.iter().join(" ");
        write!(f, "{} - {}", &self.url, tags)?;
        Ok(())
    }
}

impl PartialEq for Bookmark {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url && self.tags == other.tags
    }
}

impl Hash for Bookmark {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.url.hash(state);
        for t in &self.tags {
            t.hash(state)
        }
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
}
