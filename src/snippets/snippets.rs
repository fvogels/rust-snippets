use std::{cell::{OnceCell}, collections::{HashMap, HashSet}, rc::Rc};

use crate::{document::{self, Document, Fragment, SyntaxHighlighter, Theme}, util::attstring::{self}};
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

    #[error("Missing snippets.yaml")]
    MissingSnippetsYaml,
}

pub mod raw {
    use std::{collections::HashMap, fs, path::Path};

    use rkyv::{Archive, Deserialize, Serialize};
    use crate::{snippets::snippets::{SnippetError}, util::{attstring, segment_file}};

    #[derive(Debug, serde::Deserialize)]
    struct Metadata {
        description: String,
        tags: Option<HashMap<String, Vec<String>>>,
        keywords: Option<Vec<String>>,
    }

    #[derive(Debug, Archive, Serialize, Deserialize)]

    pub struct Snippet {
        pub description: String,
        pub parts: Vec<Part>,
        pub tags: Vec<Tag>,
        pub keywords: Vec<String>,
    }

    #[derive(Debug, Archive, Serialize, Deserialize)]
    pub struct Tag {
        pub category: String,
        pub name: String,
    }

    #[derive(Debug, Archive, Serialize, Deserialize)]
    pub struct Part {
        pub attributes: Vec<(String, String)>,
        pub source: Vec<String>,
        pub url: Option<String>,
        pub caption: Option<String>,
    }

    impl Snippet {
        pub fn parse(source: &str) -> Result<Self, SnippetError> {
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
                            attstring::parse(caption.as_str()).map_err(SnippetError::AttributeError).map(|attrs| attrs)
                        },
                        None => {
                            Ok(Vec::new())
                        }
                    }?;

                    let caption = attributes.iter().find(|pair| pair.0 == "caption").map(|p| p.1.clone());
                    let url = attributes.iter().find(|pair| pair.0 == "url").map(|p| p.1.clone());

                    let part = Part{ attributes, source: segment.lines, caption, url };

                    Ok(part)
                }).collect();
            let parts = parts?;

            if parts.len() == 0 {
                return Err(SnippetError::MissingSnippetSegments);
            }

            let tags = {
                let mut result = Vec::new();
                let tag_map = metadata.tags.unwrap_or_default();

                for (category, names) in tag_map.into_iter() {
                    for name in names {
                        let tag = Tag{ category: category.clone(), name };
                        result.push(tag);
                    }
                }

                result
            };
            // for path_component in path.iter() {
            //     tags.push(path_component.clone());
            // }

            let keywords = metadata.keywords.unwrap_or_default();

            let snippet = Snippet{
                description: metadata.description,
                parts,
                tags,
                keywords,
            };

            Ok(snippet)
        }

        pub fn load<P: AsRef<Path>>(file_path: P) -> Result<Self, SnippetError> {
            let file_path = file_path.as_ref();
            let source = fs::read_to_string(file_path).map_err(SnippetError::IoError)?;

            log::info!("Reading {}", file_path.display());
            Snippet::parse(source.as_str())
        }
    }

    fn parse_metadata(source: &str) -> Result<Metadata, SnippetError> {
        serde_yaml::from_str::<Metadata>(source).map_err(|e| SnippetError::MalformedMetadata(e))
    }
}


pub struct Snippet {
    pub description: String,
    pub parts: Vec<Part>,
    pub tags: Vec<Tag>,
    pub tag_set: HashSet<String>,
    pub extra_keywords: Vec<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Tag {
    pub category: String,
    pub name: String,
}

pub struct Part {
    pub attributes: HashMap<String, String>,
    pub syntax_highlighter: Rc<SyntaxHighlighter>,
    pub caption: Option<String>,
    pub url: Option<String>,
    pub source: Vec<String>,
    pub contents: OnceCell<Document>,
}

impl Snippet {
    pub fn from_raw(raw_snippet: raw::Snippet, syntax_highlighter: Rc<document::SyntaxHighlighter>) -> Self {
        let description = raw_snippet.description;
        let tags = raw_snippet.tags.into_iter().map(Tag::from_raw).collect::<Vec<_>>();
        let tag_set = tags.iter().map(|tag| tag.name.clone()).collect();
        let parts = raw_snippet.parts.into_iter().map(|raw_part| Part::from_raw(raw_part, syntax_highlighter.clone())).collect();
        let extra_keywords = raw_snippet.keywords;

        Snippet {
            description,
            parts,
            tags,
            tag_set,
            extra_keywords,
        }
    }

    pub fn keywords(&self) -> Vec<String> {
        let mut keywords = self.extra_keywords.clone();

        self.description.split(" ").for_each(|s| keywords.push(s.to_lowercase()));
        self.tags.iter().for_each(|tag| keywords.push(tag.name.to_lowercase()));

        keywords
    }

    pub fn has_tags<'a>(&self, tags: impl Iterator<Item=&'a str>) -> bool {
        tags.into_iter().all(|tag| self.tag_set.contains(tag))
    }
}

impl Tag {
    pub fn from_raw(raw_tag: raw::Tag) -> Self {
        Tag {
            category: raw_tag.category,
            name: raw_tag.name,
        }
    }
}

impl Part {
    pub fn from_raw(raw_part: raw::Part, syntax_highlighter: Rc<document::SyntaxHighlighter>) -> Self {
        let raw::Part { attributes, source, caption, url } = raw_part;
        let attributes = attributes.into_iter().collect();

        Part{ attributes, source, syntax_highlighter, contents: OnceCell::new(), caption, url }
    }

    pub fn document(&self) -> &document::Document {
        self.contents.get_or_init(|| {
            let lines = self.source.iter().map(|line| convert_tabs_to_spaces(line.as_str())).collect::<Vec<_>>();
            let markdown_source = lines.join("\n");
            let theme = Theme::default();
            document::parse(markdown_source.as_str(), &*self.syntax_highlighter, &theme)
        })
    }

    pub fn find_code_block_with_index(&self, index: usize) -> Option<&str> {
        let mut counter = index;

        for fragment in self.document() {
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

fn convert_tabs_to_spaces(s: &str) -> String {
    s.replace("\t", "    ")
}
