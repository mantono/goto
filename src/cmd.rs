use crate::{
    bookmark::{self, Bookmark},
    tag::Tag,
    Error,
};
use dialoguer::{console::Term, Editor, Input, Select};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashSet, io::Write, path::PathBuf, str::FromStr};
use std::{iter::FromIterator, process::ExitStatus};
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
    Open {
        #[structopt(short = "s", long = "score", default_value = "0.05")]
        min_score: f64,
        keywords: Vec<Tag>,
    },
    /// Select from a list of bookmarks
    ///
    /// Select from a list of bookmarks
    Select {
        #[structopt(short = "s", long = "score", default_value = "0.05")]
        min_score: f64,
        #[structopt(short = "n", long, default_value = "10")]
        limit: usize,
        keywords: Vec<Tag>,
    },
    /// List bookmarks
    ///
    /// List bookmarks matching the keywords, but do not open any bookmark.
    List {
        #[structopt(short = "s", long = "score", default_value = "0.05")]
        min_score: f64,
        #[structopt(short = "n", long, default_value = "10")]
        limit: usize,
        keywords: Vec<Tag>,
    },
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

fn filter(dir: &PathBuf, keywords: Vec<Tag>, min_score: f64) -> Vec<(f64, Bookmark)> {
    let keywords: HashSet<Tag> = HashSet::from_iter(keywords);
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter_map(|f| Bookmark::from_file(&f.into_path()))
        .map(|bkm| (score(&bkm.terms(), &keywords), bkm))
        .filter(|(score, _)| score >= &min_score)
        .sorted_unstable_by(|b0, b1| b0.0.partial_cmp(&b1.0).unwrap().reverse())
        .collect_vec()
}

pub fn open(
    buffer: &mut impl Write,
    dir: &PathBuf,
    keywords: Vec<Tag>,
    min_score: f64,
) -> Result<(), Error> {
    let query: String = search_query(&keywords);
    let bookmarks: Vec<(f64, Bookmark)> = filter(dir, keywords, min_score);
    let url: Url = match bookmarks.first() {
        Some((_, bookmark)) => bookmark.url(),
        None => {
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

fn search_query(terms: &Vec<Tag>) -> String {
    let query: String = terms.iter().map(|t| t.to_string()).join("+");
    format!("https://duckduckgo.com/?q={}", query)
}

pub fn list(
    buffer: &mut impl Write,
    dir: &PathBuf,
    keywords: Vec<Tag>,
    limit: usize,
    min_score: f64,
) -> Result<(), Error> {
    filter(dir, keywords, min_score)
        .iter()
        .take(limit)
        .for_each(|(score, bkm)| {
            writeln!(buffer, "{}: {:?} - {:?}", score, bkm.domain(), bkm.tags()).unwrap();
        });
    Ok(())
}

pub fn select(
    buffer: &mut impl Write,
    dir: &PathBuf,
    keywords: Vec<Tag>,
    limit: usize,
    min_score: f64,
) -> Result<(), Error> {
    let bookmarks: Vec<Bookmark> = filter(dir, keywords, min_score)
        .into_iter()
        .take(limit)
        .map(|(_, bkm)| bkm)
        .collect();

    let selection: Option<usize> = Select::new()
        .with_prompt("Select bookmark")
        .default(0)
        .items(&bookmarks)
        .interact_on_opt(&Term::stdout())?;

    match selection {
        Some(i) => select_action(buffer, dir, bookmarks[i].clone()),
        None => Ok(()),
    }
}

fn select_action(buffer: &mut impl Write, dir: &PathBuf, bookmark: Bookmark) -> Result<(), Error> {
    let actions = vec!["open", "edit tags", "edit URL", "delete", "exit"];
    let selection: Option<usize> = Select::new()
        .with_prompt("Select action")
        .default(0)
        .items(&actions)
        .interact_on_opt(&Term::stdout())?;

    match selection {
        Some(0) => {
            open::that(bookmark.url().to_string())?;
        }
        Some(1) => {
            let tags: String = bookmark.tags().iter().join(" ");
            let tags: String = Input::new()
                .with_prompt("Tags")
                .with_initial_text(tags)
                .interact_text_on(&Term::stdout())?;

            println!("New tags: {}", tags);
            let tags = Tag::new_set(tags);
            let bookmark = Bookmark::new(bookmark.url(), tags).unwrap();
            save_bookmark(dir, bookmark, false)?;
        }
        Some(2) => {
            let url: String = bookmark.url().to_string();
            let url: String = Input::new()
                .with_prompt("URL")
                .with_initial_text(url)
                .interact_text_on(&Term::stdout())?;

            let url = url.trim();

            let new_bookmark = Bookmark::new(url, bookmark.tags().clone()).unwrap();
            save_bookmark(dir, new_bookmark, true)?;
            delete_bookmark(dir, &bookmark)?;
            println!("New url {}", url);
        }
        Some(3) => {
            delete_bookmark(dir, &bookmark)?;
            let url: String = bookmark.url().to_string();
            println!("Deleted bookmark {}", url);
        }
        _ => {}
    };
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
        .join(" ");

    let tags: String = Input::new()
        .with_prompt("Tags")
        .with_initial_text(default)
        .interact_text()?;

    let tags: HashSet<Tag> = Tag::new_set(tags);
    let bkm = bookmark::Bookmark::new(url, tags).unwrap();
    let bkm: Bookmark = save_bookmark(dir, bkm, true)?;

    writeln!(buffer, "{}", bkm)?;

    Ok(())
}

fn save_bookmark(dir: &PathBuf, bkm: Bookmark, merge: bool) -> Result<Bookmark, std::io::Error> {
    let full_path = dir.join(bkm.rel_path());
    std::fs::create_dir_all(full_path.parent().unwrap())?;

    let bkm: Bookmark = if full_path.exists() && merge {
        match Bookmark::from_file(&full_path) {
            Some(prior_bkm) => bkm.merge(prior_bkm),
            None => bkm,
        }
    } else {
        bkm
    };

    let json: String = serde_json::to_string(&bkm)?;
    std::fs::write(full_path, json)?;

    Ok(bkm)
}

fn delete_bookmark(dir: &PathBuf, bkm: &Bookmark) -> Result<(), std::io::Error> {
    let full_path = dir.join(bkm.rel_path());
    std::fs::remove_file(full_path)
}
