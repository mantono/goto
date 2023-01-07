use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashSet, convert::TryInto, fmt::Display, path::PathBuf, str::FromStr};
use std::{hash::Hash, path::Path};
use url::Url;

use crate::tag::Tag;

#[derive(Eq, Serialize, Deserialize, Clone, Debug)]
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

        Ok(bm)
    }

    pub fn from_file(path: &Path) -> Result<Self, FileError> {
        if !path.exists() {
            return Err(FileError::NotFound);
        }

        if !path.is_file() {
            return Err(FileError::NotFile);
        }

        let extension: Option<&str> = match path.extension() {
            Some(ext) => ext.to_str(),
            None => None,
        };

        match extension {
            Some("yaml") => (),
            None => return Err(FileError::UnknownExtension),
            Some(ext) => return Err(FileError::UnsupportedExtension(ext.to_string())),
        };

        let bytes: Vec<u8> = std::fs::read(path)?;

        serde_yaml::from_slice(&bytes).map_err(|e| {
            println!("{} when parsing {:?}", e, path);
            FileError::Deserialize
        })
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
        let parts: Vec<&str> = self.domain()?.split('.').collect();
        parts.iter().nth_back(1).copied()
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
            Tag::new(domain).ok().map(|d| terms.insert(d));
        };
        terms
    }

    pub fn rel_path(&self) -> PathBuf {
        let domain = self.domain().unwrap_or("").to_string();
        let hash = self.id();
        let path: String = format!("{}/{}.{}", domain, hash, "yaml");
        path.into()
    }

    pub fn merge(self, other: Bookmark) -> Bookmark {
        if self.url != other.url {
            self
        } else {
            let tags: HashSet<Tag> = self.tags.iter().chain(other.tags.iter()).cloned().collect();

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

impl FromStr for Bookmark {
    type Err = serde_yaml::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_yaml::from_str(s)
    }
}

impl TryInto<Bookmark> for &[u8] {
    type Error = serde_yaml::Error;

    fn try_into(self) -> Result<Bookmark, Self::Error> {
        serde_yaml::from_slice(self)
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidUrl,
    NoDomain,
}

#[derive(Debug, Clone)]
pub enum FileError {
    NotFound,
    NotFile,
    UnknownExtension,
    UnsupportedExtension(String),
    Deserialize,
    Serialize,
    IO(String),
}

impl From<std::io::Error> for FileError {
    fn from(error: std::io::Error) -> Self {
        Self::IO(error.to_string())
    }
}

impl From<serde_yaml::Error> for FileError {
    fn from(_: serde_yaml::Error) -> Self {
        Self::Deserialize
    }
}

impl Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            FileError::NotFound => String::from("File not found"),
            FileError::NotFile => String::from("Not a file"),
            FileError::Deserialize => String::from("Unable to deserialize"),
            FileError::Serialize => String::from("Unable to serialize"),
            FileError::IO(e) => format!("IO erro: {}", e),
            FileError::UnknownExtension => String::from("Unknown extension"),
            FileError::UnsupportedExtension(ext) => match ext.as_str() {
                "json" => String::from(JSON_ERROR),
                _ => format!("Unsupported extension '{}'", ext),
            },
        };
        f.write_str(&s)
    }
}

const JSON_ERROR: &str = r#"
    JSON is no longer supported.
    Run 'goto migrate' to migrate all bookmarks files from JSON to YAML.
    See https://github.com/mantono/goto#deprecated-json-support for more information.
"#;
