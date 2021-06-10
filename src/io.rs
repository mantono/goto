use std::{
    collections::HashSet,
    convert::TryInto,
    thread::{self, JoinHandle},
    time::Duration,
};
use std::{io::Result, sync::mpsc};

use crate::tag::{Tag, TagHolder};
use crossterm::event::{self, Event, KeyEvent};
use dialoguer::{console::Term, theme::Theme, Input};
use url::Url;

pub fn read_title(default: Option<String>, theme: &dyn Theme) -> Option<String> {
    Input::with_theme(theme)
        .with_prompt("Title")
        .allow_empty(true)
        .with_initial_text(default.unwrap_or_default())
        .interact_text_on(&Term::stdout())
        .ok()
}

pub fn read_tags(default: impl TagHolder, theme: &dyn Theme) -> HashSet<Tag> {
    let tags: Result<String> = Input::with_theme(theme)
        .with_prompt("Tags")
        .allow_empty(true)
        .with_initial_text(default.join())
        .interact_text_on(&Term::stdout());

    match tags {
        Ok(tags) => Tag::new_set(tags),
        Err(_) => default.tags(),
    }
}

pub fn read_url(default: Url, theme: &dyn Theme) -> Url {
    let url: Option<String> = Input::with_theme(theme)
        .with_prompt("URL")
        .allow_empty(false)
        .with_initial_text(default.to_string())
        .interact_text_on(&Term::stdout())
        .ok();

    match url {
        Some(url) => match url.trim().try_into() {
            Ok(url) => url,
            Err(_) => read_url(default, theme),
        },
        None => default,
    }
}

pub fn poll_events(tx: mpsc::Sender<Event>) -> JoinHandle<()> {
    let tick_rate = Duration::from_millis(2000);
    thread::spawn(move || loop {
        let has_event: bool = match event::poll(tick_rate) {
            Ok(status) => status,
            Err(e) => {
                log::error!("Unable to read event: {}", e);
                break;
            }
        };
        if !has_event {
            continue;
        }
        match event::read() {
            Ok(event) => match tx.send(event) {
                Ok(_) => {
                    log::debug!("Event sent")
                }
                Err(e) => {
                    log::info!("Could not send event: {:?}", e);
                    break;
                }
            },
            Err(err) => panic!("Gott error {}", err),
        }
    })
}
