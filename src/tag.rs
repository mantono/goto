use std::collections::HashSet;

use lazy_static::lazy_static;
use regex::Regex;
use std::{fmt::Display, hash::Hash, str::FromStr};

lazy_static! {
    static ref TERMINATOR: Regex = Regex::new(r"[,\s]+").unwrap();
    static ref DISCARD: Regex = Regex::new(r#"[,\s"\\]+"#).unwrap();
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
pub struct Tag(String);

impl Tag {
    pub fn new<T: Into<String>>(tag: T) -> Result<Tag, TagError> {
        let tag: String = Self::normalize(&tag.into());
        if tag.is_empty() {
            Err(TagError::Empty)
        } else {
            Ok(Tag(tag))
        }
    }

    pub fn value(&self) -> String {
        self.0.clone()
    }

    fn normalize(input: &str) -> String {
        DISCARD
            .replace_all(input, "")
            .to_lowercase()
            .trim()
            .to_string()
    }

    pub fn new_set<T: Into<String>>(tags: T) -> HashSet<Tag> {
        TERMINATOR
            .split(&tags.into())
            .filter_map(|t| Tag::from_str(t).ok())
            .collect()
    }
}

#[derive(Debug)]
pub enum TagError {
    Empty,
}

impl Display for TagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Tag was empty or contained no valid characters")
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Tag {
    type Err = TagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Tag::new(s)
    }
}

impl AsRef<String> for Tag {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl Hash for Tag {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}