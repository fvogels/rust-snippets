use ratatui::{buffer::Buffer, layout::Rect, widgets::{List, ListState, StatefulWidget}};


pub struct TreeView<'a, T> where T: Tree {
    tree: &'a T
}

pub struct TreeViewState {
    list_state: ListState,
}

pub trait Tree {
    type Node;

    fn root(&self) -> Self::Node;
    fn children(&self, parent: &Self::Node) -> Vec<Self::Node>;
    fn caption<'a>(&self, node: &'a Self::Node) -> &'a str;
}

impl TreeViewState {
    pub fn new() -> Self {
        TreeViewState{
            list_state: ListState::default(),
        }
    }
}

impl<'a, T> TreeView<'a, T> where T: Tree {
    pub fn new(tree: &'a T) -> Self {
        TreeView{
            tree,
        }
    }
}

impl<'a, T> StatefulWidget for TreeView<'a, T> where T: Tree {
    type State = TreeViewState;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut TreeViewState) {
        let lines = convert_tree_to_list(self.tree);
        let list = List::new(lines);

        StatefulWidget::render(list, area, buffer, &mut state.list_state);
    }
}

fn convert_tree_to_list<T>(tree: &T) -> Vec<String> where T: Tree {
    let mut lines = Vec::new();
    let root = tree.root();
    let mut stack = vec![(0, root)];

    while let Some((depth, node)) = stack.pop() {
        if depth > 0 {
            let mut line = " ".repeat(depth - 1);
            line.push_str(tree.caption(&node));
            lines.push(line);
        }

        let mut children = tree.children(&node);
        children.reverse();

        for child in children.into_iter() {
            let child_depth = depth + 1;
            stack.push((child_depth, child))
        }
    }

    lines
}
