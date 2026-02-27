use crate::snippets::{Folder, Hierarchy, Library};

pub struct TreeAdapter<'a> {
    hierarchy: &'a Hierarchy
}

impl<'a> TreeAdapter<'a> {
    pub fn new(hierarchy: &'a Hierarchy) -> Self {
        TreeAdapter{
            hierarchy,
        }
    }
}

impl<'a> super::widgets::tree_view::Tree for TreeAdapter<'a> {
    type Node = (String, &'a Folder);

    fn root(&self) -> Self::Node {
        ("".to_owned(), self.hierarchy.root())
    }

    fn children(&self, parent: &Self::Node) -> Vec<Self::Node> {
        let mut result = Vec::new();

        for (name, subfolder) in parent.1.subfolders() {
            result.push((name.clone(), subfolder));
        }

        result
    }

    fn caption<'b>(&self, node: &'b Self::Node) -> &'b str {
        node.0.as_str()
    }
}
