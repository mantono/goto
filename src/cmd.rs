use crate::{
    bookmark::{self, Bookmark, FileError},
    io::{self, Streams},
    tag::{Tag, TagHolder},
    Error,
};
use clap::{Subcommand, ValueEnum};
use dialoguer::{theme::Theme, FuzzySelect, Select};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::{BTreeSet, HashSet},
    io::Write,
    thread::{self, JoinHandle},
};
use std::{
    iter::FromIterator,
    path::{Path, PathBuf},
};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum FormatField {
    Url,
    Title,
    Tags,
    Path,
}

impl std::str::FromStr for FormatField {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "url" => Ok(FormatField::Url),
            "title" => Ok(FormatField::Title),
            "tags" => Ok(FormatField::Tags),
            "path" => Ok(FormatField::Path),
            other => Err(format!("unknown format field: {}", other)),
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Add bookmark with URL
    ///
    /// Add bookmark with URL and optionally some tags
    Add { url: String, tags: Vec<Tag> },
    /// Edit an existing bookmark
    ///
    /// Edit the existing bookmark YAML file, and validate that the file has correct YAML syntax,
    /// before overwriting the previous version of the file.
    Edit { path: PathBuf },
    /// List bookmarks
    ///
    /// List bookmarks containing all of the provided tags. If no tags are provided, all bookmarks
    /// are listed. Default output: URL only (one per line). Use --format to customise fields.
    /// Accepted fields: url, title, tags, path. Example: goto list --format=url,title,tags,path
    /// Use --delimiter to set the separator between fields (default: |, no surrounding spaces).
    List {
        #[clap(long, value_delimiter = ',')]
        format: Vec<FormatField>,
        #[clap(long, default_value = "|")]
        delimiter: String,
        tags: Vec<Tag>,
    },
    /// Migrate format of bookmarks
    ///
    /// Migrate all existing bookmarks from JSON to YAML. This action is not reversible.
    #[cfg(feature = "migrate")]
    Migrate,
}

impl Default for Command {
    fn default() -> Self {
        Command::List {
            format: vec![FormatField::Url],
            delimiter: String::from("|"),
            tags: Vec::new(),
        }
    }
}

lazy_static! {
    static ref PROTOCOL_PREFIX: Regex = regex::Regex::new("^https?://").unwrap();
    static ref TITLE: Regex =
        regex::Regex::new(r"<(title|TITLE)>\s?.*\s?</(title|TITLE)>").unwrap();
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name().to_str().map(|f| f.starts_with('.')).unwrap_or(false)
}

pub fn add(
    mut streams: Streams,
    dir: &Path,
    url: String,
    default: impl TagHolder,
    theme: &dyn Theme,
) -> Result<(), Error> {
    let url: String = if PROTOCOL_PREFIX.is_match(&url) { url } else { format!("https://{}", url) };
    let url = url::Url::parse(&url).unwrap();
    let title: JoinHandle<Option<String>> = load_title(&url);
    let tags: BTreeSet<Tag> = io::read_tags(default, theme, streams.term());
    let loaded_title: Option<String> = title.join().unwrap_or_default();
    let title: Option<String> = io::read_title(loaded_title, theme, streams.term());

    let bkm = bookmark::Bookmark::new(url, title, tags).unwrap();
    let bkm: Bookmark = save_bookmark(dir, bkm, true)?;

    writeln!(streams.output(), "{}", bkm)?;

    Ok(())
}

fn load_title(url: &Url) -> JoinHandle<Option<String>> {
    let url = url.clone();
    thread::spawn(move || {
        let body: String = reqwest::blocking::get(url).unwrap().text().unwrap();
        let title: String = TITLE.find(&body).map(|title| title.as_str().to_string())?;
        let title = title
            .chars()
            .skip(7)
            .take_while(|c| *c != '<')
            .collect::<String>()
            .trim()
            .to_string();

        Some(title)
    })
}

fn save_bookmark(dir: &Path, bkm: Bookmark, merge: bool) -> Result<Bookmark, FileError> {
    let full_path = dir.join(bkm.rel_path());
    std::fs::create_dir_all(full_path.parent().expect("Create full path"))?;

    let bkm: Bookmark = if full_path.exists() && merge {
        Bookmark::from_file(&full_path).map(|prior_bkm| bkm.merge(prior_bkm))?
    } else {
        bkm
    };

    let yaml: String = serde_yaml::to_string(&bkm).map_err(|_| FileError::Serialize)?;
    std::fs::write(full_path, yaml)?;

    Ok(bkm)
}

pub fn edit(mut streams: Streams, path: PathBuf, theme: &dyn Theme) -> Result<(), Error> {
    let bookmark = Bookmark::from_file(&path)?;
    let title = io::read_title(bookmark.title(), theme, streams.term());
    let tags = io::read_tags(bookmark.tags().clone(), theme, streams.term());
    let updated = Bookmark::new(bookmark.url(), title, tags).unwrap();
    let yaml = serde_yaml::to_string(&updated).map_err(|_| FileError::Serialize)?;
    std::fs::write(&path, yaml)?;
    writeln!(streams.output(), "{}", updated)?;
    Ok(())
}

fn delete_bookmark(dir: &Path, bkm: &Bookmark) -> Result<(), std::io::Error> {
    let full_path = dir.join(bkm.rel_path());
    std::fs::remove_file(full_path)
}

fn format_list_line(
    fields: &[FormatField],
    bkm: &Bookmark,
    path: &Path,
    delimiter: &str,
) -> String {
    if fields.is_empty() {
        return bkm.url().to_string();
    }
    fields
        .iter()
        .map(|field| match field {
            FormatField::Url => bkm.url().to_string(),
            FormatField::Title => bkm.title().unwrap_or_default(),
            FormatField::Tags => bkm.tags().iter().sorted().join(" "),
            FormatField::Path => path.display().to_string(),
        })
        .join(delimiter)
}

fn has_all_tags(bkm: &Bookmark, tags: &[Tag]) -> bool {
    tags.iter().all(|t| bkm.tags().contains(t))
}

pub fn list(
    mut streams: Streams,
    dir: &Path,
    tags: Vec<Tag>,
    format: Vec<FormatField>,
    delimiter: String,
) -> Result<(), Error> {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_entry(|f| !is_hidden(f))
        .filter_map(|f| f.ok())
        .filter(|f| f.file_type().is_file())
        .filter_map(|f| {
            let path: PathBuf = f.into_path();
            match Bookmark::from_file(&path) {
                Ok(bkm) => Some((path, bkm)),
                Err(e) => {
                    log::error!("Unable to read {}: {}", path.to_str().unwrap_or_default(), e);
                    None
                }
            }
        })
        .filter(|(_, bkm)| has_all_tags(bkm, &tags))
        .try_for_each(|(path, bkm)| {
            writeln!(streams.output(), "{}", format_list_line(&format, &bkm, &path, &delimiter))
        })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bookmark(url: &str, title: Option<&str>, tags: &[&str]) -> Bookmark {
        let url: url::Url = url::Url::parse(url).unwrap();
        let tags: BTreeSet<Tag> = tags.iter().filter_map(|t| Tag::new(*t).ok()).collect();
        Bookmark::new(url, title.map(String::from), tags).unwrap()
    }

    #[test]
    fn format_default_empty_fields_returns_url_only() {
        let bkm: Bookmark = make_bookmark("https://example.com", Some("Title"), &["rust"]);
        let path: &Path = Path::new("/data/goto/example.com/abc.yaml");
        let line: String = format_list_line(&[], &bkm, path, "|");
        assert_eq!(line, "https://example.com/");
    }

    #[test]
    fn format_url_field_returns_url() {
        let bkm: Bookmark = make_bookmark("https://example.com", None, &[]);
        let path: &Path = Path::new("/data/goto/example.com/abc.yaml");
        let line: String = format_list_line(&[FormatField::Url], &bkm, path, "|");
        assert_eq!(line, "https://example.com/");
    }

    #[test]
    fn format_url_title_tags_with_title_present() {
        let bkm: Bookmark =
            make_bookmark("https://example.com", Some("My Site"), &["docs", "rust"]);
        let path: &Path = Path::new("/data/goto/example.com/abc.yaml");
        let line: String = format_list_line(
            &[FormatField::Url, FormatField::Title, FormatField::Tags],
            &bkm,
            path,
            "|",
        );
        assert_eq!(line, "https://example.com/|My Site|docs rust");
    }

    #[test]
    fn format_title_empty_string_when_absent() {
        let bkm: Bookmark = make_bookmark("https://example.com", None, &["rust"]);
        let path: &Path = Path::new("/data/goto/example.com/abc.yaml");
        let line: String = format_list_line(
            &[FormatField::Url, FormatField::Title, FormatField::Tags],
            &bkm,
            path,
            "|",
        );
        assert_eq!(line, "https://example.com/||rust");
    }

    #[test]
    fn format_path_field_returns_absolute_path() {
        let bkm: Bookmark = make_bookmark("https://example.com", None, &[]);
        let path: &Path = Path::new("/data/goto/example.com/abc.yaml");
        let line: String = format_list_line(&[FormatField::Path], &bkm, path, "|");
        assert_eq!(line, "/data/goto/example.com/abc.yaml");
    }

    #[test]
    fn format_fields_respect_declared_order() {
        let bkm: Bookmark = make_bookmark("https://example.com", Some("T"), &["z", "a"]);
        let path: &Path = Path::new("/p/x.yaml");
        let line: String =
            format_list_line(&[FormatField::Tags, FormatField::Url], &bkm, path, "|");
        assert_eq!(line, "a z|https://example.com/");
    }

    #[test]
    fn format_custom_delimiter_is_used_verbatim() {
        let bkm: Bookmark = make_bookmark("https://example.com", Some("T"), &[]);
        let path: &Path = Path::new("/p/x.yaml");
        let line: String =
            format_list_line(&[FormatField::Url, FormatField::Title], &bkm, path, " | ");
        assert_eq!(line, "https://example.com/ | T");
    }

    #[test]
    fn has_all_tags_returns_true_when_all_present() {
        let bkm: Bookmark = make_bookmark("https://example.com", None, &["rust", "docs", "async"]);
        let required: Vec<Tag> = vec![Tag::new("rust").unwrap(), Tag::new("docs").unwrap()];
        assert!(has_all_tags(&bkm, &required));
    }

    #[test]
    fn has_all_tags_returns_false_when_tag_missing() {
        let bkm: Bookmark = make_bookmark("https://example.com", None, &["rust"]);
        let required: Vec<Tag> = vec![Tag::new("rust").unwrap(), Tag::new("docs").unwrap()];
        assert!(!has_all_tags(&bkm, &required));
    }

    #[test]
    fn has_all_tags_returns_true_when_no_tags_required() {
        let bkm: Bookmark = make_bookmark("https://example.com", None, &["rust"]);
        let required: Vec<Tag> = vec![];
        assert!(has_all_tags(&bkm, &required));
    }
}
