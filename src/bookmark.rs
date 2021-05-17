use itertools::Itertools;
use sha2::{Digest, Sha256};
use std::hash::Hash;
use std::{collections::HashSet, convert::TryInto, fmt::Display, path::PathBuf};
use url::Url;

use crate::tag::Tag;

#[derive(PartialEq, Eq)]
pub struct Bookmark {
    url: Url,
    tags: HashSet<Tag>,
}

impl Bookmark {
    pub fn new<T: TryInto<Url>>(url: T, tags: HashSet<Tag>) -> Result<Self, Error> {
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
        let tags: HashSet<Tag> = Tag::new_set(*content.last()?);

        Bookmark::new(url, tags).ok()
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
        let mut hash = hash(&self.url.to_string());
        hash.push_str(".bkm");
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
        write!(f, "{}\n", &self.url)?;
        let tags: String = self.tags.iter().join(" ");
        write!(f, "{}\n", &tags)?;
        Ok(())
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
