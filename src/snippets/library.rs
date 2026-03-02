use std::{collections::HashSet, fs::{self, File}, io::Read, path::Path};

use rkyv::{api::low::{deserialize, from_bytes}, rancor::Error, to_bytes};

use crate::{snippets::{snippets::{Snippet, SnippetError, load_snippets}}, util::trie};

pub struct Library {
    snippets: Vec<Snippet>,
    trie: trie::Trie<usize>,
    tags: Vec<String>,
}

impl Library {
    pub fn new(snippets: impl Iterator<Item=Snippet>) -> Self {
        let mut snippets = snippets.collect::<Vec<_>>();
        snippets.sort_by(|x, y| x.description.cmp(&y.description));

        let mut trie_builder = trie::Builder::new();

        for snippet_index in 0..snippets.len() {
            let snippet = &snippets[snippet_index];
            for keyword in snippet.keywords() {
                trie_builder.add(&keyword, snippet_index);
            }
        }

        let library = Library{
            tags: collect_tags(snippets.iter()),
            snippets: snippets,
            trie: trie_builder.finalize(),
        };

        library
    }

    pub fn load_files<P>(root: &P) -> Result<Self, SnippetError>
    where P: AsRef<Path> {
        let snippets = load_snippets(root)?;
        let library = Library::new(snippets.into_iter());

        Ok(library)
    }

    pub fn read_archive<P>(archive_path: &P) -> Result<Self, SnippetError>
    where P: AsRef<Path> {
        let data = fs::read(archive_path).map_err(SnippetError::IoError)?;
        let bytes = &data[..];

        let snippets = from_bytes::<Vec<Snippet>, Error>(bytes).map_err(|_| SnippetError::SerializationError)?;
        Ok(Self::new(snippets.into_iter()))
    }

    pub fn write_to_archive<P>(&self, archive_path: &P) -> Result<(), SnippetError>
    where P: AsRef<Path> {
        let snippets = &self.snippets;
        let bytes = to_bytes::<rkyv::rancor::Error>(snippets).map_err(|_| SnippetError::SerializationError)?;

        fs::write(archive_path, bytes).map_err(SnippetError::IoError)?;

        Ok(())
    }

    pub fn search<'a, 'b>(&self, keywords: impl Iterator<Item=&'a str>, tags: impl Iterator<Item=&'b str>) -> Vec<usize> {
        let mut intersection = self.collect_snippets_with_tags(tags);

        for keyword in keywords {
            if !keyword.is_empty() { // small optimization
                let set = self.search_single(keyword);
                intersection.retain(|i| set.contains(i));
            }
        }

        let mut result: Vec<usize> = intersection.iter().copied().collect();
        result.sort();
        result
    }

    fn collect_snippets_with_tags<'a>(&self, tags: impl Iterator<Item=&'a str>) -> HashSet<usize> {
        let tag_list = tags.collect::<Vec<_>>();

        (0..self.snippets.len()).filter(|id| self.snippet(*id).has_tags(tag_list.iter().copied())).collect()
    }

    fn search_single(&self, keyword: &str) -> HashSet<usize> {
        let mut result = HashSet::new();

        self.trie.iter(keyword).for_each(|snippet_indices| {
            snippet_indices.iter().copied().for_each(|snippet_index| { result.insert(snippet_index); } )
        });

        result
    }

    pub fn snippets(&self) -> impl Iterator<Item=usize> {
        0..self.snippets.len()
    }

    pub fn snippet<'a>(&'a self, index: usize) -> &'a Snippet {
        &self.snippets[index]
    }

    pub fn tags<'a>(&'a self) -> &'a Vec<String> {
        &self.tags
    }
}

fn collect_tags<'a>(snippets: impl Iterator<Item=&'a Snippet>) -> Vec<String> {
    let mut tag_set = HashSet::new();

    for snippet in snippets {
        for tag in snippet.tags.iter() {
            tag_set.insert(tag.clone());
        }
    }

    let mut tags = tag_set.into_iter().collect::<Vec<_>>();
    tags.sort();

    tags
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::snippets::{Library, snippets::Snippet};

    #[test]
    fn search_with_tags() {
        let snippet1 = Snippet{
            description: String::from("a"),
            parts: Vec::new(),
            path: Vec::new(),
            tags: HashSet::from([String::from("tag-a"), String::from("tag-b")]),
        };

        let library = Library::new(vec![snippet1].into_iter());

        let found_snippets = library.search(Vec::new().into_iter(), Vec::new().into_iter());

        assert_eq!(found_snippets.len(), 1);
    }

    #[test]
    fn search_with_tags_zero_results() {
        let snippet1 = Snippet{
            description: String::from("a"),
            parts: Vec::new(),
            path: Vec::new(),
            tags: HashSet::from([String::from("tag-a"), String::from("tag-b")]),
        };

        let library = Library::new(vec![snippet1].into_iter());

        let found_snippets = library.search(Vec::new().into_iter(), vec![String::from("tag-x").as_str()].into_iter());

        assert_eq!(found_snippets.len(), 0);
    }
}