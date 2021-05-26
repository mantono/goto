use std::io::Result;
use std::{collections::HashSet, convert::TryInto};

use crate::tag::Tag;
use dialoguer::{console::Term, Input};
use itertools::Itertools;
use url::Url;

pub fn read_title(default: Option<String>) -> Option<String> {
    Input::new()
        .with_prompt("Title")
        .allow_empty(true)
        .with_initial_text(default.unwrap_or_default())
        .interact_text_on(&Term::stdout())
        .ok()
}

pub fn read_tags(default: HashSet<Tag>) -> HashSet<Tag> {
    let tags: String = default.iter().join(" ");
    let tags: Result<String> = Input::new()
        .with_prompt("Tags")
        .with_initial_text(tags)
        .interact_text_on(&Term::stdout());

    match tags {
        Ok(tags) => Tag::new_set(tags),
        Err(_) => default,
    }
}

pub fn read_url(default: Option<Url>) -> Url {
    let fallback = default.clone();
    let initial = default.map(|u| u.to_string()).unwrap_or("".to_string());
    let url: Option<String> = Input::new()
        .with_prompt("URL")
        .allow_empty(false)
        .with_initial_text(initial)
        .interact_text_on(&Term::stdout())
        .ok();

    match url {
        Some(url) => url.trim().try_into().unwrap(),
        None => fallback.unwrap(),
    }
}
