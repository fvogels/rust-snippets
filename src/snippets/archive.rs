use std::{fs, path::{Path, PathBuf}};

use rkyv::{from_bytes, to_bytes, rancor::Error};
use walkdir::WalkDir;

use crate::snippets::snippets::{SnippetError, raw};

pub struct Archive {
    pub raw_snippets: Vec<raw::Snippet>,
}

impl Archive {
    fn new(raw_snippets: Vec<raw::Snippet>) -> Self {
        Archive { raw_snippets }
    }

    pub fn load_snippet_files<P>(root: &P) -> Result<Self, SnippetError> where P: AsRef<Path> {
        let absolute_root = root.as_ref().canonicalize().map_err(|_| SnippetError::PathError)?;
        let raw_snippets: Result<Vec<raw::Snippet>, SnippetError> = discover_files(root)?.into_iter().map(|file_path| {
            let path = derive_path(&absolute_root, &file_path)?;
            raw::Snippet::load(file_path, path)
        }).collect();
        let raw_snippets = raw_snippets?;

        Ok(Archive { raw_snippets })
    }

    pub fn load<P>(archive_path: &P) -> Result<Self, SnippetError> where P: AsRef<Path> {
        let data = fs::read(archive_path).map_err(SnippetError::IoError)?;
        let bytes = &data[..];

        let snippets = from_bytes::<Vec<raw::Snippet>, Error>(bytes).map_err(|_| SnippetError::SerializationError)?;
        Ok(Self::new(snippets))
    }

    pub fn write<P>(&self, archive_path: &P) -> Result<(), SnippetError> where P: AsRef<Path> {
        let snippets = &self.raw_snippets;
        let bytes = to_bytes::<rkyv::rancor::Error>(snippets).map_err(|_| SnippetError::SerializationError)?;

        fs::write(archive_path, bytes).map_err(SnippetError::IoError)?;

        Ok(())
    }
}

fn discover_files<P>(root: &P) -> Result<Vec<PathBuf>, SnippetError>
where P: AsRef<Path> {
    WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.metadata().unwrap().is_file())
            .map(|e| e.path().canonicalize().map_err(SnippetError::IoError).map(|p| p.to_owned()))
            .collect()
}

fn derive_path<P, Q>(root: &P, file: &Q) -> Result<Vec<String>, SnippetError> where P: AsRef<Path>, Q: AsRef<Path> {
    debug_assert!(root.as_ref().is_absolute(), "{} must be absolute", root.as_ref().as_os_str().display());
    debug_assert!(file.as_ref().is_absolute(), "{} must be absolute", file.as_ref().as_os_str().display());

    let parent_path = file.as_ref().parent().ok_or(SnippetError::PathError)?;

    let path =
        parent_path.strip_prefix(root)
                   .map_err(|_| SnippetError::PathError)?
                   .components()
                   .map(|component| component.as_os_str().to_str().unwrap().to_owned())
                   .collect::<Vec<String>>();

    Ok(path)
}
