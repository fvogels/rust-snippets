use std::{collections::HashMap, fs, path::Path};

use rkyv::{from_bytes, to_bytes, rancor::Error};
use serde::Deserialize;

use crate::snippets::snippets::{SnippetError, raw};

pub struct Archive {
    pub raw_snippets: Vec<raw::Snippet>,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    tags: Option<HashMap<String, Vec<String>>>,
}

impl Metadata {
    fn tags(&self) -> Vec<raw::Tag> {
        let mut result = Vec::new();

        if let Some(tags) = &self.tags {
            for entry in tags {
                let category = entry.0;
                let names = entry.1.clone();

                for name in names {
                    let tag = raw::Tag{ category: category.clone(), name };
                    result.push(tag);
                }
            }
        }

        result
    }
}

impl Archive {
    fn new(raw_snippets: Vec<raw::Snippet>) -> Self {
        Archive { raw_snippets }
    }

    pub fn load_snippet_files<P: AsRef<Path>>(root: &P) -> Result<Self, SnippetError> {
        let mut raw_snippets = Vec::new();

        Archive::load_snippet_files_rec(root, &mut |raw_snippet| raw_snippets.push(raw_snippet))?;

        let archive = Archive { raw_snippets };

        Ok(archive)
    }

    fn load_snippet_files_rec<P: AsRef<Path>>(directory: &P, receiver: &mut dyn FnMut(raw::Snippet)) -> Result<(), SnippetError> {
        let directory = directory.as_ref();

        let metadata_file_path = directory.join("_metadata.yaml");
        if !metadata_file_path.exists() {
            return Err(SnippetError::MissingSnippetsYaml)
        }

        let metadata = fs::read_to_string(metadata_file_path)?;
        let metadata = serde_yaml::from_str::<Metadata>(&metadata).unwrap();

        for entry in directory.read_dir().map_err(SnippetError::IoError)? {
            let entry = entry.map_err(SnippetError::IoError)?;
            let is_directory = entry.file_type().map_err(SnippetError::IoError)?.is_dir();

            if is_directory {
                Archive::load_snippet_files_rec(&entry.path(), &mut |mut raw_snippet| {
                    for tag in metadata.tags() {
                        raw_snippet.tags.push(tag);
                    }

                    receiver(raw_snippet);
                })?;
            }
            else if entry.file_name().to_str().unwrap().ends_with(".snippet") {
                let mut snippet = raw::Snippet::load(entry.path())?;

                for tag in metadata.tags() {
                    snippet.tags.push(tag);
                }

                receiver(snippet);
            }
        }

        Ok(())
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
