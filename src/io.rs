use std::io::{BufWriter, Stderr, Stdout, Write};
use std::{collections::HashSet, convert::TryInto};

use crate::tag::{Tag, TagHolder};
use dialoguer::{console::Term, theme::Theme, Input};
use url::Url;

pub struct Streams {
    ui: BufWriter<Stderr>,
    output: BufWriter<Stdout>,
    term: Term,
}

impl Streams {
    pub fn new() -> Self {
        Streams {
            ui: std::io::BufWriter::new(std::io::stderr()),
            output: std::io::BufWriter::new(std::io::stdout()),
            term: Term::stderr(),
        }
    }

    pub fn ui(&mut self) -> &mut BufWriter<Stderr> {
        &mut self.ui
    }

    pub fn output(&mut self) -> &mut BufWriter<Stdout> {
        &mut self.output
    }

    pub fn term(&self) -> &Term {
        &self.term
    }

    pub fn flush_all(&mut self) -> Result<(), std::io::Error> {
        self.ui.flush()?;
        self.output.flush()?;
        Ok(())
    }
}

impl Drop for Streams {
    fn drop(&mut self) {
        if let Err(e) = self.flush_all() {
            log::warn!("Unable to flush buffers: {:?}", e)
        }
    }
}

pub fn read_title(default: Option<String>, theme: &dyn Theme, term: &Term) -> Option<String> {
    Input::with_theme(theme)
        .with_prompt("Title")
        .allow_empty(true)
        .with_initial_text(default.unwrap_or_default())
        .interact_text_on(term)
        .ok()
}

pub fn read_tags(default: impl TagHolder, theme: &dyn Theme, term: &Term) -> HashSet<Tag> {
    let tags: std::io::Result<String> = Input::with_theme(theme)
        .with_prompt("Tags")
        .allow_empty(true)
        .with_initial_text(default.join())
        .interact_text_on(term);

    match tags {
        Ok(tags) => Tag::new_set(tags),
        Err(_) => default.tags(),
    }
}

pub fn read_url(default: Url, theme: &dyn Theme, term: &Term) -> Url {
    let url: Option<String> = Input::with_theme(theme)
        .with_prompt("URL")
        .allow_empty(false)
        .with_initial_text(default.to_string())
        .interact_text_on(term)
        .ok();

    match url {
        Some(url) => match url.trim().try_into() {
            Ok(url) => url,
            Err(_) => read_url(default, theme, term),
        },
        None => default,
    }
}
