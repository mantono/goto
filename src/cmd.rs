use crate::{
    bookmark::{self, Bookmark},
    tag::Tag,
    Error,
};
use dialoguer::Input;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::iter::FromIterator;
use std::{collections::HashSet, io::Write, path::PathBuf};
use url::Url;

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Add bookmark with URL
    ///
    /// Add bookmark with URL and optionally some tags
    Add { url: String, tags: Vec<Tag> },
    /// Open bookmark in browser
    ///
    /// Open a bookmark in the browser that is matching the given keywords. If several bookmarks
    /// match the keywords, the best matching bookmark will be selected. If no bookmark is matching
    /// the keywords, the keywords will be directed to a search query in a search engine.
    Open { keywords: Vec<Tag> },
    /// Select from a list of bookmarks
    ///
    /// Select from a list of bookmarks where at least one keyword matches the domain or a tag of
    /// each bookmark in the list.
    Select {
        #[structopt(short = "n", long, default_value = "10")]
        limit: usize,
        keywords: Vec<Tag>,
    },
    /// List bookmarks
    ///
    /// List bookmarks matching the keywords, but do not open any bookmark.
    List { keywords: Vec<Tag> },
    /// Edit a bookmark
    ///
    /// Edit a bookmark in your editor of choice
    Edit { path: PathBuf },
    /// Delete bookmark
    Delete { path: PathBuf },
}

lazy_static! {
    static ref PROTOCOL_PREFIX: Regex = regex::Regex::new("^https?://").unwrap();
}

pub fn open(buffer: &mut impl Write, dir: &PathBuf, keywords: Vec<Tag>) -> Result<(), Error> {
    let keywords: HashSet<Tag> = HashSet::from_iter(keywords);
    let bkm: Option<Bookmark> = walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter_map(|f| Bookmark::from_file(&f.into_path()))
        .map(|bkm| (score(&bkm.terms(), &keywords), bkm))
        .inspect(|(score, bkm)| println!("{}, {}", score, bkm))
        .filter(|(score, _)| score > &0.0)
        .sorted_unstable_by(|b0, b1| b0.0.partial_cmp(&b1.0).unwrap().reverse())
        .map(|(_, bkm)| bkm)
        .next();

    let url: Url = match bkm {
        Some(bookmark) => bookmark.url(),
        None => {
            let query: String = keywords.iter().map(|t| t.to_string()).join("+");
            let query = format!("https://duckduckgo.com/?q={}", query);
            writeln!(
                buffer,
                "No bookmark found for keyword(s), searching online instead"
            )
            .unwrap();
            Url::parse(&query).unwrap()
        }
    };
    open::that(url.to_string()).unwrap();

    Ok(())
}

fn score(v0: &HashSet<Tag>, v1: &HashSet<Tag>) -> f64 {
    let union: Vec<&Tag> = v0.union(&v1).collect();
    let intersection: Vec<&Tag> = v0.intersection(&v1).collect();

    let union: f64 = union.len() as f64;
    let intersection: f64 = intersection.len() as f64;
    intersection / union
}

pub fn add(
    buffer: &mut impl Write,
    dir: &PathBuf,
    url: String,
    tags: Vec<Tag>,
) -> Result<(), Error> {
    let url: String = if PROTOCOL_PREFIX.is_match(&url) {
        url
    } else {
        format!("https://{}", url)
    };
    let url = url::Url::parse(&url).unwrap();
    let default: String = tags
        .iter()
        .map(|t| t.value())
        .collect::<Vec<String>>()
        .join(", ");

    let tags: String = Input::new()
        .with_prompt("Tags")
        .default(default)
        .interact_text()?;

    let tags: HashSet<Tag> = Tag::new_set(tags);
    let bkm = bookmark::Bookmark::new(url, tags).unwrap();

    let full_path = dir.join(bkm.rel_path());
    std::fs::create_dir_all(full_path.parent().unwrap())?;

    let bkm: Bookmark = if full_path.exists() {
        match Bookmark::from_file(&full_path) {
            Some(prior_bkm) => bkm.merge(prior_bkm),
            None => bkm,
        }
    } else {
        bkm
    };

    std::fs::write(full_path, bkm.to_string())?;

    writeln!(buffer, "{}", bkm)?;

    Ok(())
}