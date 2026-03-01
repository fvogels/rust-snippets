use std::{collections::HashSet, path::Path};

use itertools::enumerate;

use crate::{snippets::{hierarchy::Hierarchy, snippets::{Snippet, SnippetError, load_snippets}}, util::trie};

pub struct Library {
    snippets: Vec<Snippet>,
    trie: trie::Trie<usize>,
    tags: Vec<String>,
}

impl Library {
    pub fn load<P>(root: &P) -> Result<Library, SnippetError>
    where P: AsRef<Path> {
        let mut snippets = load_snippets(root)?;
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

        Ok(library)
    }

    pub fn search<'a, 'b, 'c>(&'a self, keywords: impl Iterator<Item=&'b str>, tags: impl Iterator<Item=&'c str>) -> Vec<usize> {
        let mut intersection = self.collect_snippets_with_tags(tags);

        for index in 0..self.snippets.len() {
            intersection.insert(index);
        }

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