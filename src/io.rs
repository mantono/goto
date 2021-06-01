use std::io::Result;
use std::{collections::HashSet, convert::TryInto};

use crate::tag::{Tag, TagHolder};
use dialoguer::{console::Term, Input};
use url::Url;

pub fn read_title(default: Option<String>) -> Option<String> {
    Input::new()
        .with_prompt("Title")
        .allow_empty(true)
        .with_initial_text(default.unwrap_or_default())
        .interact_text_on(&Term::stdout())
        .ok()
}

pub fn read_tags(default: impl TagHolder) -> HashSet<Tag> {
    let tags: Result<String> = Input::new()
        .with_prompt("Tags")
        .allow_empty(true)
        .with_initial_text(default.join())
        .interact_text_on(&Term::stdout());

    match tags {
        Ok(tags) => Tag::new_set(tags),
        Err(_) => default.tags(),
    }
}

pub fn read_url(default: Url) -> Url {
    let url: Option<String> = Input::new()
        .with_prompt("URL")
        .allow_empty(false)
        .with_initial_text(default.to_string())
        .interact_text_on(&Term::stdout())
        .ok();

    match url {
        Some(url) => match url.trim().try_into() {
            Ok(url) => url,
            Err(_) => read_url(default),
        },
        None => default,
    }
}
