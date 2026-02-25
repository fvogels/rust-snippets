use std::{collections::HashSet, path::Path};

use crate::{snippets::snippets::{Snippet, SnippetError, load_snippets}, util::trie};

pub struct Library {
    snippets: Vec<Snippet>,
    trie: trie::Trie<usize>,
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
            snippets: snippets,
            trie: trie_builder.finalize(),
        };

        Ok(library)
    }

    pub fn search<'a, 'b>(&'a self, keywords: impl Iterator<Item=&'b str>) -> Vec<&'a Snippet> {
        let mut intersection = HashSet::new();

        for index in 0..self.snippets.len() {
            intersection.insert(index);
        }

        for keyword in keywords {
            let set = self.search_single(keyword);
            intersection.retain(|i| set.contains(i));
        }

        let result: Vec<&'a Snippet> = intersection.iter().copied().map(|i| &self.snippets[i]).collect();
        // TODO sort result
        result
    }

    fn search_single(&self, keyword: &str) -> HashSet<usize> {
        let mut result = HashSet::new();

        self.trie.iter(keyword).for_each(|snippet_indices| {
            snippet_indices.iter().copied().for_each(|snippet_index| { result.insert(snippet_index); } )
        });

        result
    }

    pub fn snippets<'a>(&'a self) -> &'a Vec<Snippet> {
        &self.snippets
    }
}
