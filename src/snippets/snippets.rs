use std::{path::{Path, PathBuf}};

use walkdir::WalkDir;
use crate::util::segment_file;
use thiserror::Error;
use std::io;
use serde::{Deserialize};


#[derive(Debug, Error)]
pub enum SnippetError
{
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Missing metadata segment in {0}")]
    MissingMetadataSegment(PathBuf),

    #[error("Missing snippet segments in {0}")]
    MissingSnippetSegments(PathBuf),

    #[error("YAML error: {0}")]
    MalformedMetadata(#[from] serde_yaml::Error),

    #[error("Badly structured metadata in {0}: {1}")]
    BadlyStructuredMetadata(PathBuf, String),
}

#[derive(Debug, Deserialize)]
struct Metadata
{
    description: String,
    language: String,
    tags: Vec<String>,
}

#[derive(Debug)]
pub struct Snippet
{
    pub description: String,
    pub language: String,
    pub parts: Vec<Part>,
}

#[derive(Debug)]
pub struct Part
{
    pub caption: String,
    pub lines: Vec<String>,
}

impl Snippet {
    pub fn keywords(&self) -> Vec<String> {
        let mut keywords = Vec::new();

        keywords.push(self.language.to_lowercase());
        // TODO remove accents, better splitting
        self.description.split(" ").for_each(|s| keywords.push(s.to_lowercase()));

        keywords
    }
}

pub fn discover_files<P>(root: &P) -> Result<Vec<PathBuf>, SnippetError>
where P: AsRef<Path>
{
    let result = WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.metadata().unwrap().is_file())
            .map(|e| -> Result<PathBuf, SnippetError> {
                let canonical = e.path().canonicalize()?;
                Ok(PathBuf::from(canonical))
            }).collect::<Result<Vec<PathBuf>, SnippetError>>()?;

    Ok(result)
}

pub fn load_snippet_file<P>(file_path: &P) -> Result<Snippet, SnippetError>
where P: AsRef<Path>
{
    let segments = segment_file::load(file_path, |line| {
        line.strip_prefix("===").map(|x| x.trim())
    })?;

    let mut segment_iterator = segments.into_iter();
    let metadata_segment = segment_iterator.next().ok_or(SnippetError::MissingMetadataSegment(PathBuf::from(file_path.as_ref())))?;

    let metadata_string = metadata_segment.lines.join("\n");
    let metadata = parse_metadata(&metadata_string)?;

    let snippet = Snippet{
        description: metadata.description,
        language: metadata.language,
        parts: segment_iterator.map(|segment| {
            Part{
                caption: segment.caption,
                lines: segment.lines,
            }
        }).collect(),
    };

    if snippet.parts.len() == 0 {
        return Err(SnippetError::MissingSnippetSegments(PathBuf::from(file_path.as_ref())));
    }

    Ok(snippet)
}

fn parse_metadata(source: &str) -> Result<Metadata, SnippetError>
{
    serde_yaml::from_str(source).map_err(|e| SnippetError::MalformedMetadata(e))
}

pub fn load_snippets<P>(root: &P) -> Result<Vec<Snippet>, SnippetError>
where P: AsRef<Path>
{
    discover_files(root)?.into_iter().map(|path| load_snippet_file(&path)).collect()
}
