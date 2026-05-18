use std::{collections::{HashMap, HashSet}, rc::Rc};

use crate::{document, snippets::{archive::Archive, snippets::{Snippet, Tag}}, util::trie};

pub struct Library {
    snippets: Vec<Snippet>,
    trie: trie::Trie<usize>,
    tags: Vec<Tag>,
}

impl Library {
    pub fn new(snippets: impl Iterator<Item=Snippet>) -> Self {
        let snippets = snippets.collect::<Vec<_>>();
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

    pub fn from_archive(archive: Archive, syntax_highlighter: Rc<document::SyntaxHighlighter>) -> Self {
        let raw_snippets = {
            let mut snippets = archive.raw_snippets;
            snippets.sort_by(|x, y| x.description.cmp(&y.description));
            snippets
        };
        let mut snippet_table = HashMap::new();

        for (index, raw_snippet) in raw_snippets.iter().enumerate() {
            if let Some(id) = &raw_snippet.identifier {
                snippet_table.insert(id.clone(), index);
            }
        }

        let snippets = raw_snippets.into_iter().map(|raw_snippet| Snippet::from_raw(raw_snippet, &snippet_table, syntax_highlighter.clone()));
        Library::new(snippets)
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

    pub fn tags<'a>(&'a self) -> &'a Vec<Tag> {
        &self.tags
    }
}

fn collect_tags<'a>(snippets: impl Iterator<Item=&'a Snippet>) -> Vec<Tag> {
    let mut tag_set = HashSet::new();

    for snippet in snippets {
        for tag in snippet.tags.iter() {
            tag_set.insert(tag.clone());
        }
    }

    let mut tags = tag_set.into_iter().collect::<Vec<_>>();
    tags.sort_by(|tag1, tag2| tag1.name.to_lowercase().cmp(&tag2.name.to_lowercase()));

    tags
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::snippets::{Library, snippets::{Snippet, Tag}};

    fn create_feature_tag(name: &str) -> Tag {
        Tag { category: "feature".to_owned(), name: name.to_owned() }
    }

    #[test]
    fn search_with_tags() {
        let snippet1 = Snippet{
            description: String::from("a"),
            extra_keywords: vec![],
            pages: Vec::new(),
            links: Vec::new(),
            tags: vec![create_feature_tag("tag-a"), create_feature_tag("tag-b")],
            tag_set: HashSet::from([String::from("tag-a"), String::from("tag-b")]),
        };

        let library = Library::new(vec![snippet1].into_iter());

        let found_snippets = library.search(Vec::new().into_iter(), Vec::new().into_iter());

        assert_eq!(found_snippets.len(), 1);
    }

    #[test]
    fn search_with_tags_zero_results() {
        let snippet1 = Snippet{
            description: String::from("a"),
            extra_keywords: vec![],
            pages: Vec::new(),
            links: Vec::new(),
            tags: vec![create_feature_tag("tag-a"), create_feature_tag("tag-b")],
            tag_set: HashSet::from([String::from("tag-a"), String::from("tag-b")]),
        };

        let library = Library::new(vec![snippet1].into_iter());

        let found_snippets = library.search(Vec::new().into_iter(), vec![String::from("tag-x").as_str()].into_iter());

        assert_eq!(found_snippets.len(), 0);
    }
}