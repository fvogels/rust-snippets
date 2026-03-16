use std::{collections::{HashMap, HashSet}, path::{Path, PathBuf}};
use rkyv::{Archive, Deserialize, Serialize};

use walkdir::WalkDir;
use crate::{document::{self, Document, Fragment, Theme}, util::{attstring::{self, Attributes}, segment_file}};
use thiserror::Error;
use std::io;


#[derive(Debug, Error)]
pub enum SnippetError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Missing metadata segment")]
    MissingMetadataSegment,

    #[error("Missing snippet segments")]
    MissingSnippetSegments,

    #[error("Path error")]
    PathError,

    #[error("Serialization error")]
    SerializationError,

    #[error("YAML error: {0}")]
    MalformedMetadata(#[from] serde_yaml::Error),

    #[error("Attribute error: {0}")]
    AttributeError(#[from] attstring::Error),
}

#[derive(Debug, serde::Deserialize)]
struct Metadata {
    description: String,
    tags: Vec<String>,
}

pub mod raw {
    use std::{fs, path::Path};

    use rkyv::{Archive, Deserialize, Serialize};
    use crate::{snippets::snippets::SnippetError, util::{attstring, segment_file}};

    #[derive(Debug, serde::Deserialize)]
    struct Metadata {
        description: String,
        tags: Vec<String>,
    }

    #[derive(Debug, Archive, Serialize, Deserialize)]

    pub struct Snippet {
        pub description: String,
        pub parts: Vec<Part>,
        pub tags: Vec<String>,
        pub path: Vec<String>,
    }

    #[derive(Debug, Archive, Serialize, Deserialize)]
    pub struct Part {
        pub attributes: Vec<(String, String)>,
        pub source: Vec<String>,
    }

    impl Snippet {
        pub fn parse(source: &str, path: Vec<String>) -> Result<Self, SnippetError> {
            let segments = segment_file::parse(source.lines(), |line| {
                line.strip_prefix("===").map(|x| x.trim())
            });

            let mut segment_iterator = segments.into_iter();
            let metadata_segment = segment_iterator.next().ok_or(SnippetError::MissingMetadataSegment)?;

            let metadata_string = metadata_segment.lines.join("\n");
            let metadata = parse_metadata(&metadata_string)?;

            let parts: Result<Vec<Part>, SnippetError> =  segment_iterator.map(|segment| {
                    let attributes = match segment.caption {
                        Some(caption) => {
                            attstring::parse(caption.as_str()).map_err(SnippetError::AttributeError).map(|attrs| attrs.pairs())
                        },
                        None => {
                            Ok(Vec::new())
                        }
                    }?;

                    let part = Part{ attributes, source: segment.lines };

                    Ok(part)
                }).collect();
            let parts = parts?;

            if parts.len() == 0 {
                return Err(SnippetError::MissingSnippetSegments);
            }

            let mut tags = metadata.tags;
            for path_component in path.iter() {
                tags.push(path_component.clone());
            }

            let snippet = Snippet{
                description: metadata.description,
                parts: parts,
                tags: tags,
                path: path,
            };

            Ok(snippet)
        }

        pub fn load<P>(file_path: P, path: Vec<String>) -> Result<Self, SnippetError> where P: AsRef<Path> {
            let source = fs::read_to_string(file_path).map_err(SnippetError::IoError)?;

            Snippet::parse(source.as_str(), path)
        }
    }

    fn parse_metadata(source: &str) -> Result<Metadata, SnippetError> {
        serde_yaml::from_str::<Metadata>(source).map_err(|e| SnippetError::MalformedMetadata(e))
    }
}


#[derive(Debug, Archive, Serialize, Deserialize)]
pub struct Snippet {
    pub description: String,
    pub parts: Vec<Part>,
    pub tags: HashSet<String>,
    pub path: Vec<String>,
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub struct Part {
    pub attributes: HashMap<String, String>,
    pub contents: Document,
}

impl Snippet {
    pub fn from_raw(raw_snippet: raw::Snippet, syntax_highlighter: &document::SyntaxHighlighter) -> Self {
        let description = raw_snippet.description;
        let tags = raw_snippet.tags.into_iter().collect();
        let path = raw_snippet.path;
        let parts = raw_snippet.parts.into_iter().map(|raw_part| Part::from_raw(raw_part, syntax_highlighter)).collect();

        Snippet {
            description,
            parts,
            tags,
            path
        }
    }

    pub fn keywords(&self) -> Vec<String> {
        let mut keywords = Vec::new();

        self.description.split(" ").for_each(|s| keywords.push(s.to_lowercase()));
        self.tags.iter().for_each(|tag| keywords.push(tag.to_lowercase()));

        keywords
    }

    pub fn has_tags<'a>(&self, tags: impl Iterator<Item=&'a str>) -> bool {
        tags.into_iter().all(|tag| self.tags.contains(tag))
    }
}

impl Part {
    pub fn from_raw(raw_part: raw::Part, syntax_highlighter: &document::SyntaxHighlighter) -> Self {
        let attributes = {
            let mut result = HashMap::new();
            for (key, value) in raw_part.attributes {
                result.insert(key, value);
            }
            result
        };

        let contents = {
            let lines = raw_part.source.iter().map(|line| convert_tabs_to_spaces(line.as_str())).collect::<Vec<_>>();
            let markdown_source = lines.join("\n");
            let theme = Theme::default();
            document::parse(markdown_source.as_str(), &syntax_highlighter, &theme)
        };

        Part{ attributes, contents }
    }

    pub fn language(&self) -> Option<&str> {
        self.attributes.get("language").map(String::as_str)
    }

    pub fn caption(&self) -> Option<&str> {
        self.attributes.get("caption").map(String::as_str)
    }

    pub fn find_code_block_with_index(&self, index: usize) -> Option<&str> {
        let mut counter = index;

        for fragment in &self.contents {
            if let Fragment::Code { original, .. } = fragment {
                if counter == 0 {
                    return Some(original.as_str())
                }
                else {
                    counter -= 1;
                }
            }
        }

        None
    }
}

pub fn discover_files<P>(root: &P) -> Result<Vec<PathBuf>, SnippetError>
where P: AsRef<Path> {
    WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.metadata().unwrap().is_file())
            .map(|e| e.path().canonicalize().map_err(SnippetError::IoError).map(|p| p.to_owned()))
            .collect()
}

pub fn load_snippet_file<P, Q>(root_path: &P, file_path: &Q, syntax_highlighter: &document::SyntaxHighlighter) -> Result<Snippet, SnippetError>
where P: AsRef<Path>, Q: AsRef<Path> {
    let path = derive_path(root_path, file_path)?;
    let raw_snippet = raw::Snippet::load(file_path, path)?;
    let snippet = Snippet::from_raw(raw_snippet, syntax_highlighter);

    Ok(snippet)
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

pub fn load_snippets<P>(root: &P, syntax_highlighter: &document::SyntaxHighlighter) -> Result<Vec<Snippet>, SnippetError> where P: AsRef<Path> {
    let absolute_root = root.as_ref().canonicalize().map_err(|_| SnippetError::PathError)?;

    discover_files(root)?.into_iter().map(|path| load_snippet_file(&absolute_root, &path, syntax_highlighter)).collect()
}

fn convert_tabs_to_spaces(s: &str) -> String {
    s.replace("\t", "    ")
}
