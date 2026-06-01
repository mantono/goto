use crate::{
    bookmark::{self, Bookmark, FileError},
    io::{self, Streams},
    tag::{Tag, TagHolder},
    Error,
};
use clap::{Subcommand, ValueEnum};
use dialoguer::theme::Theme;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::BTreeSet,
    io::Write,
    path::{Path, PathBuf},
};

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
    let url: url::Url = url::Url::parse(&url).unwrap();
    let cli_tags: BTreeSet<Tag> = default.tags();
    let bare: Bookmark = bookmark::Bookmark::new(url.clone(), None, cli_tags).unwrap();
    let suggested: Bookmark = run_on_add_hook(bare)?;
    let tags: BTreeSet<Tag> = io::read_tags(suggested.tags().clone(), theme, streams.term());
    let title: Option<String> = io::read_title(suggested.title(), theme, streams.term());

    let bkm: Bookmark = bookmark::Bookmark::new(url, title, tags).unwrap();
    let bkm: Bookmark = save_bookmark(dir, bkm, true)?;

    writeln!(streams.output(), "{}", bkm)?;

    Ok(())
}

#[derive(Debug)]
pub enum HookError {
    NonZeroExit(i32, String),
    InvalidOutput(String),
    Io(std::io::Error),
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookError::NonZeroExit(code, stderr) => {
                write!(f, "on-add hook exited with code {}: {}", code, stderr)
            }
            HookError::InvalidOutput(msg) => {
                write!(f, "on-add hook returned invalid YAML: {}", msg)
            }
            HookError::Io(e) => write!(f, "on-add hook I/O error: {}", e),
        }
    }
}

impl From<std::io::Error> for HookError {
    fn from(e: std::io::Error) -> Self {
        HookError::Io(e)
    }
}

impl From<HookError> for Error {
    fn from(e: HookError) -> Self {
        Error::HookFailed(e.to_string())
    }
}

fn hook_path() -> PathBuf {
    dirs_next::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("goto")
        .join("hooks")
        .join("on-add")
}

fn run_on_add_hook(bkm: Bookmark) -> Result<Bookmark, HookError> {
    run_on_add_hook_with_path(bkm, &hook_path())
}

fn run_on_add_hook_with_path(bkm: Bookmark, path: &Path) -> Result<Bookmark, HookError> {
    use std::io::Write as _;
    use std::process::{Command, Stdio};

    if !path.exists() {
        return Ok(bkm);
    }

    let yaml: String =
        serde_yaml::to_string(&bkm).map_err(|e| HookError::InvalidOutput(e.to_string()))?;

    let mut child = Command::new(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    child.stdin.take().unwrap().write_all(yaml.as_bytes())?;

    let output = child.wait_with_output()?;

    if !output.status.success() {
        let code: i32 = output.status.code().unwrap_or(-1);
        let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(HookError::NonZeroExit(code, stderr));
    }

    let modified: Bookmark = serde_yaml::from_slice(&output.stdout)
        .map_err(|e| HookError::InvalidOutput(e.to_string()))?;

    Ok(modified)
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
    use std::os::unix::fs::PermissionsExt;

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

    // --- on-add hook tests ---

    fn write_hook(dir: &Path, script: &str) -> PathBuf {
        let hook_path: PathBuf = dir.join("on-add");
        std::fs::write(&hook_path, script).unwrap();
        std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o755)).unwrap();
        hook_path
    }

    #[test]
    fn hook_absent_returns_bookmark_unchanged() {
        let bkm: Bookmark = make_bookmark("https://example.com", Some("Title"), &["rust"]);
        let non_existent: PathBuf = PathBuf::from("/tmp/goto_hook_does_not_exist_xyz");
        let result: Bookmark = run_on_add_hook_with_path(bkm.clone(), &non_existent).unwrap();
        assert_eq!(result, bkm);
    }

    #[test]
    fn hook_passthrough_returns_bookmark_unchanged() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let hook_path: PathBuf = write_hook(dir.path(), "#!/bin/sh\ncat\n");
        let bkm: Bookmark = make_bookmark("https://example.com", Some("Title"), &["rust"]);
        let result: Bookmark = run_on_add_hook_with_path(bkm.clone(), &hook_path).unwrap();
        assert_eq!(result, bkm);
    }

    #[test]
    fn hook_can_modify_bookmark_title() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        // Hook that sets the title field in the YAML output
        let hook_path: PathBuf = write_hook(
            dir.path(),
            "#!/bin/sh\nprintf 'url: https://example.com/\\ntitle: Injected Title\\ntags:\\n  - rust\\n'\n",
        );
        let bkm: Bookmark = make_bookmark("https://example.com", None, &["rust"]);
        let result: Bookmark = run_on_add_hook_with_path(bkm, &hook_path).unwrap();
        assert_eq!(result.title(), Some("Injected Title".to_string()));
    }

    #[test]
    fn hook_non_zero_exit_returns_error() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let hook_path: PathBuf = write_hook(dir.path(), "#!/bin/sh\nexit 1\n");
        let bkm: Bookmark = make_bookmark("https://example.com", None, &[]);
        let result: Result<Bookmark, HookError> = run_on_add_hook_with_path(bkm, &hook_path);
        assert!(matches!(result, Err(HookError::NonZeroExit(1, _))));
    }

    #[test]
    fn hook_invalid_yaml_output_returns_error() {
        let dir: tempfile::TempDir = tempfile::tempdir().unwrap();
        let hook_path: PathBuf =
            write_hook(dir.path(), "#!/bin/sh\nprintf 'not: valid: yaml: :::'\n");
        let bkm: Bookmark = make_bookmark("https://example.com", None, &[]);
        let result: Result<Bookmark, HookError> = run_on_add_hook_with_path(bkm, &hook_path);
        assert!(matches!(result, Err(HookError::InvalidOutput(_))));
    }
}
