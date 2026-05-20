use std::{collections::{HashMap, HashSet}, fs, path::Path};

use anyhow::Context;
use rkyv::{from_bytes, to_bytes, rancor::Error};
use serde::Deserialize;
use anyhow;

use crate::snippets::snippets::raw;

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

    pub fn load_snippet_files<P: AsRef<Path>>(root: &P) -> anyhow::Result<Self> {
        let root = root.as_ref();
        let mut raw_snippets = Vec::new();
        Archive::load_snippet_files_rec(&root, &mut |raw_snippet| raw_snippets.push(raw_snippet)).with_context(|| format!("Loading all snippet files recursively, starting in {}", root.display()))?;

        Archive::verify(&raw_snippets)?;

        let archive = Archive { raw_snippets };
        Ok(archive)
    }

    fn verify(raw_snippets: &Vec<raw::Snippet>) -> anyhow::Result<()> {
        Archive::verify_tag_categorization(raw_snippets)?;

        let snippet_identifiers = Archive::collect_identifiers(raw_snippets)?;
        Archive::verify_link_existence(&snippet_identifiers, raw_snippets)?;

        Ok(())
    }

    fn collect_identifiers(snippets: &Vec<raw::Snippet>) -> anyhow::Result<HashSet<String>> {
        let mut identifiers = HashMap::new();

        for snippet in snippets {
            if let Some(identifier) = &snippet.identifier {
                if let Some(path) = identifiers.get(identifier) {
                    anyhow::bail!("{} and {} have the same identifier", path, snippet.path);
                }

                identifiers.insert(identifier.clone(), snippet.path.clone());
            }
        }

        Ok(identifiers.into_keys().collect::<HashSet<_>>())
    }

    fn verify_link_existence(valid_identifiers: &HashSet<String>, snippets: &Vec<raw::Snippet>) -> anyhow::Result<()> {
        for snippet in snippets {
            for link in &snippet.links {
                if !valid_identifiers.contains(link) {
                    anyhow::bail!("Snippet {} links to invalid identifier {}", snippet.path, link);
                }
            }
        }

        Ok(())
    }

    fn verify_tag_categorization(snippets: &Vec<raw::Snippet>) -> anyhow::Result<()> {
        let mut category_table: HashMap<String, String> = HashMap::new();
        let mut snippet_table: HashMap<String, Vec<&raw::Snippet>> = HashMap::new();

        for snippet in snippets {
            for tag in &snippet.tags {
                if let Some(category) = category_table.get(&tag.name) {
                    if *category != tag.category {
                        let contradicting_snippets = &snippet_table[&tag.name];

                        log::error!("Inconsistent tag category: tag {} has categories {} and {}", tag.name, category, tag.category);

                        for snippet in contradicting_snippets {
                            log::error!("Snippet {} categorizes it as {}", snippet.path, category);
                        }
                        log::error!("Snippet {} categorized it as {}", snippet.path, tag.category);

                        anyhow::bail!("Inconsistent tag categories; see log for details");
                    }
                }
                else {
                    category_table.insert(tag.name.clone(), tag.category.clone());
                    snippet_table.entry(tag.name.clone()).or_default().push(&snippet);
                }
            }
        }

        Ok(())
    }

    fn load_snippet_files_rec<P: AsRef<Path>>(directory: &P, receiver: &mut dyn FnMut(raw::Snippet)) -> anyhow::Result<()> {
        let directory = directory.as_ref();
        let metadata_filename = "_metadata.yaml";

        let metadata_file_path = directory.join(metadata_filename);
        if !metadata_file_path.exists() {
            anyhow::bail!("Could not find {} in directory {}", metadata_filename, directory.display());
        }

        let metadata = fs::read_to_string(metadata_file_path)?;
        let metadata = serde_yaml::from_str::<Metadata>(&metadata).context("failed to parse yaml")?;

        for entry in directory.read_dir()? {
            let entry = entry?;
            let is_directory = entry.file_type()?.is_dir();

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

    pub fn load<P: AsRef<Path>>(archive_path: &P) -> anyhow::Result<Self> {
        let archive_path = archive_path.as_ref();
        let data = fs::read(archive_path).with_context(|| format!("Loading binary archive {}", archive_path.display()))?;
        let bytes = &data[..];

        let snippets = from_bytes::<Vec<raw::Snippet>, Error>(bytes).context("Deserializing archive")?;
        Ok(Self::new(snippets))
    }

    pub fn write<P: AsRef<Path>>(&self, archive_path: &P) -> anyhow::Result<()> {
        let archive_path = archive_path.as_ref();
        let snippets = &self.raw_snippets;
        let bytes = to_bytes::<rkyv::rancor::Error>(snippets).context("Serializing archive")?;

        fs::write(archive_path, bytes).with_context(|| format!("writing archive to {}", archive_path.display()))?;

        Ok(())
    }
}
