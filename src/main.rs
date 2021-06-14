#[macro_use]
extern crate structopt;

mod bookmark;
mod cfg;
mod cmd;
mod dbg;
mod io;
mod logger;
mod tag;

use crate::cfg::Config;
use crate::dbg::dbg_info;
use crate::logger::setup_logging;
use bookmark::Bookmark;
use crossterm::cursor::MoveTo;
use crossterm::cursor::Show;
use crossterm::event::KeyModifiers;
use crossterm::ExecutableCommand;
//use dialoguer::{console::Style, theme::Theme};
use crossterm::{
    event::{Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use dialoguer::console::colors_enabled_stderr;
use std::fmt::Display;
use std::ops::Range;
use std::{
    alloc::System,
    collections::{HashSet, VecDeque},
    io::{Stdout, Write},
    str,
    sync::mpsc::{self, Receiver},
    thread::{self, sleep, JoinHandle},
    time::Duration,
};
use std::{path::PathBuf, process};
use structopt::StructOpt;
use tag::Tag;
use tui::layout::Direction;
use tui::text::Spans;
use tui::text::Text;
use tui::widgets::Paragraph;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};

fn main() -> Result<(), Error> {
    let cfg: Config = Config::from_args();
    setup_logging(cfg.verbosity_level);

    enable_raw_mode().expect("Expceted to use raw mode");
    let out = std::io::stdout();
    let backend = CrosstermBackend::new(out);
    let mut term = Terminal::new(backend)?;
    term.set_cursor(0, 0)?;
    term.show_cursor()?;
    term.clear()?;
    let state = State::new(term);

    let (tx, rx) = mpsc::channel();
    log::info!("Launching threads");
    tx.send(Event::Key(KeyEvent::new(KeyCode::Null, KeyModifiers::NONE)))
        .unwrap();
    let handle_tx = io::poll_events(tx);
    let handle_rx = process_events(rx, state);
    handle_rx.join().unwrap();
    handle_tx.join().unwrap();
    disable_raw_mode().expect("Expected to disable raw mode");

    Ok(())
}

fn process_events(rx: Receiver<Event>, mut state: State) -> JoinHandle<()> {
    let timeout = Duration::from_millis(500);
    thread::spawn(move || loop {
        match rx.recv_timeout(timeout) {
            Ok(event) => {
                state.push_event(Some(event));
            }
            Err(e) => match e {
                mpsc::RecvTimeoutError::Timeout => {
                    state.push_event(None);
                }
                mpsc::RecvTimeoutError::Disconnected => {
                    log::info!("Receving channel disconnected");
                    break;
                }
            },
        }
    })
}

trait Filterable {
    fn matches(content: &str) -> bool;
}

struct State {
    input: TextInput,
    selected: usize,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

struct TextInput {
    input: VecDeque<char>,
    cursor: usize,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            input: VecDeque::new(),
            cursor: 0,
        }
    }
}

impl Display for TextInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let line: String = self.input.iter().collect();
        write!(f, "{}", line)
    }
}

impl TextInput {
    fn process_action(&mut self, action: Action) {
        match action {
            Action::InsertChar(c) => {
                self.input.insert(self.cursor, c);
                self.cursor += 1;
            }
            Action::MoveCursor(mv) => match mv {
                Move::Start => self.cursor = 0,
                Move::End => self.cursor = self.input.len(),
                Move::Left => self.cursor = self.cursor.saturating_sub(1),
                Move::Right => {
                    self.cursor = self.cursor.saturating_add(1).clamp(0, self.input.len())
                }
            },
            Action::DeleteLeft => {
                if self.cursor > 0 {
                    self.input.remove(self.cursor - 1);
                    self.cursor -= 1;
                }
            }
            Action::DeleteRight => {
                if self.cursor < self.input.len() {
                    self.input.remove(self.cursor);
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Action {
    InsertChar(char),
    MoveCursor(Move),
    DeleteLeft,
    DeleteRight,
    SelectionUp,
    SelectionDown,
    SelectionFirst,
    SelectionLast,
    Submit,
    Tick,
    Exit,
}

#[derive(Debug, Clone, Copy)]
enum Move {
    Start,
    End,
    Left,
    Right,
}

impl State {
    pub fn new(terminal: Terminal<CrosstermBackend<Stdout>>) -> State {
        State {
            input: TextInput::default(),
            selected: 0,
            terminal,
        }
    }

    pub fn push_event(&mut self, event: Option<Event>) -> bool {
        let action: Option<Action> = match event {
            Some(Event::Key(event)) => State::conv_key_event(event),
            _ => None,
        };

        match action {
            Some(action) => {
                self.process_action(action);
                true
            }
            None => {
                self.process_action(Action::Tick);
                false
            }
        }
    }

    fn conv_key_event(event: KeyEvent) -> Option<Action> {
        match event.code {
            KeyCode::Backspace => Some(Action::DeleteLeft),
            KeyCode::Enter => Some(Action::Submit),
            KeyCode::Left => Some(Action::MoveCursor(Move::Left)),
            KeyCode::Right => Some(Action::MoveCursor(Move::Right)),
            KeyCode::Up => Some(Action::SelectionUp),
            KeyCode::Down => Some(Action::SelectionDown),
            KeyCode::Home => Some(Action::MoveCursor(Move::Start)),
            KeyCode::End => Some(Action::MoveCursor(Move::End)),
            KeyCode::PageUp => Some(Action::SelectionFirst),
            KeyCode::PageDown => Some(Action::SelectionLast),
            KeyCode::Tab => None,
            KeyCode::BackTab => None,
            KeyCode::Delete => Some(Action::DeleteRight),
            KeyCode::Insert => None,
            KeyCode::F(_) => None,
            KeyCode::Char(c) => Self::char_action(c, event.modifiers),
            KeyCode::Null => None,
            KeyCode::Esc => Some(Action::Exit),
        }
    }

    fn char_action(c: char, modifiers: KeyModifiers) -> Option<Action> {
        if modifiers.contains(KeyModifiers::CONTROL) {
            None
        } else {
            Some(Action::InsertChar(c))
        }
    }

    fn process_action(&mut self, action: Action) {
        match action {
            Action::InsertChar(_)
            | Action::MoveCursor(_)
            | Action::DeleteLeft
            | Action::DeleteRight => self.input.process_action(action),
            Action::SelectionUp => {
                self.selected = self.selected.saturating_sub(1);
            }
            Action::SelectionDown => {
                self.selected = self.selected.saturating_add(1);
            }
            Action::SelectionFirst => {
                self.selected = 0;
            }
            Action::SelectionLast => {
                self.selected = usize::MAX;
            }
            Action::Submit => {
                //println!("{}\r", self.line());
            }
            Action::Exit => {
                log::info!("Exiting");
                process::exit(0)
            }
            Action::Tick => {}
        }
        self.render().expect("Render failed");
    }

    fn list_state(&self) -> ListState {
        let mut list = ListState::default();
        list.select(Some(self.selected));
        list
    }

    pub fn line(&self) -> String {
        self.input.to_string()
    }

    fn render(&mut self) -> Result<(), std::io::Error> {
        let mut list_state = self.list_state();
        let line: String = self.line();
        let cursor: u16 = self.input.cursor as u16;
        self.terminal.draw(|f| {
            let rect = f.size();
            f.set_cursor(cursor + 1, 1);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(rect);

            let items = [
                ListItem::new("FOO"),
                ListItem::new("BAR"),
                ListItem::new("123"),
            ];
            let list = List::new(items)
                .block(Block::default().title("bookmarks").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Green),
                )
                .highlight_symbol(">>");

            let mut text = Text::from(Spans::from(line));
            let msg =
                Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("goto"));
            f.render_widget(msg, chunks[0]);
            //f.render_stateful_widget(block, chunks[0], &mut list_state);
            f.render_stateful_widget(list, chunks[1], &mut list_state);
            //f.render_widget(list, chunks[0]);
        })?;
        Ok(())
    }

    pub fn tags(&self) -> HashSet<Tag> {
        Tag::new_set::<String>(self.line())
    }
    /* pub fn wait_next(&mut self) -> Result<(), crossterm::ErrorKind> {
        let event = crossterm::event::read()?;
        match event {
            Event::Key(key) => {
                if key.code == KeyCode::Delete {
                    process::exit(0);
                }
            }

            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
        self.events.push_back(event);
        Ok(())
    } */
}

impl Drop for State {
    fn drop(&mut self) {
        self.terminal.clear().ok();
    }
}

#[derive(Debug)]
pub enum Error {
    NotExistingFile,
    Formatting,
    Serialization,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        log::error!("{}", e.to_string());
        Self::NotExistingFile
    }
}

impl From<std::fmt::Error> for Error {
    fn from(e: std::fmt::Error) -> Self {
        log::error!("{}", e.to_string());
        Self::Formatting
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        log::error!("{}", e.to_string());
        Self::Serialization
    }
}

fn dir() -> Option<PathBuf> {
    match dirs_next::data_dir() {
        Some(mut dir) => {
            dir.push("goto");
            Some(dir)
        }
        None => match dirs_next::home_dir() {
            Some(mut dir) => {
                dir.push(".goto");
                Some(dir)
            }
            None => None,
        },
    }
}
