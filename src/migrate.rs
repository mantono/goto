use crate::{bookmark::FileError, io::Streams, Error};
use std::io::Write;
use std::path::Path;

pub fn migrate(mut streams: Streams, dir: &std::path::PathBuf) -> Result<(), Error> {
    let sum: usize = walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.file_name().to_str().unwrap_or_default().ends_with(".json"))
        .inspect(|f| writeln!(streams.ui(), "Migrating {:?}", f.path()).unwrap())
        .filter(|p| json_to_yaml(p.path()).is_ok())
        .count();

    writeln!(streams.ui(), "Migrated {} bookmarks from JSON to YAML", sum).unwrap();
    Ok(())
}

fn json_to_yaml(path: &Path) -> Result<&Path, FileError> {
    #[derive(serde::Deserialize, serde::Serialize)]
    struct Data {
        pub url: String,
        pub title: Option<String>,
        pub tags: Vec<String>,
    }

    if !path.exists() {
        return Err(FileError::NotFound);
    }

    if !path.is_file() {
        return Err(FileError::NotFile);
    }

    let extension: Option<&str> = match path.extension() {
        Some(ext) => ext.to_str(),
        None => None,
    };

    let extension: Extension = match extension {
        Some("json") => Extension::Json,
        Some("yaml") => Extension::Yaml,
        None => return Err(FileError::UnknownExtension),
        Some(ext) => return Err(FileError::UnsupportedExtension(ext.to_string())),
    };

    match extension {
        Extension::Json => {
            let bytes: Vec<u8> = std::fs::read(path)?;
            let mut data: Data = serde_json::from_slice(&bytes)?;
            data.tags.sort();
            let yaml: String = serde_yaml::to_string(&data)?;
            let target: std::path::PathBuf = path.with_extension("yaml");
            std::fs::write(target, yaml)?;
            std::fs::remove_file(path)?;
            Ok(path)
        }
        Extension::Yaml => Ok(path),
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Extension {
    Json,
    Yaml,
}

impl std::fmt::Display for Extension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Extension::Json => f.write_str("json"),
            Extension::Yaml => f.write_str("yaml"),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        log::error!("{}", e.to_string());
        Self::Serialization
    }
}

impl From<serde_json::Error> for FileError {
    fn from(_: serde_json::Error) -> Self {
        Self::Deserialize
    }
}
