pub trait Tree {
    type Node;

    fn root(&self) -> Option<&Self::Node>;
    fn children<'a>(&'a self, node: &'a Self::Node) -> impl Iterator<Item=&'a Self::Node>;
}

pub fn euler_traversal<'a, T, F>(tree: &'a T, callback: &mut F) where T: Tree, F: FnMut(&'a T::Node) {
    match tree.root() {
        Some(root) => euler_traversal_helper(tree, root, callback),
        None => {},
    }
}

fn euler_traversal_helper<'a, T, F>(tree: &'a T, current: &'a T::Node, callback: &mut F) where T: Tree, F: FnMut(&'a T::Node) {
    callback(current);

    let children = tree.children(current);
    for child in children {
        euler_traversal_helper(tree,child, callback);
        callback(current)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::util::tree_traversal::euler_traversal;

    struct TestTree {
        root: i32,
        table: HashMap<i32, Vec<i32>>,
    }

    impl TestTree {
        fn new() -> TestTree {
            TestTree { root: 0, table: HashMap::new() }
        }

        fn link(&mut self, parent: i32, children: Vec<i32>) {
            self.table.insert(parent, children);
        }
    }

    impl super::Tree for TestTree {
        type Node = i32;

        fn root(&self) -> Option<&Self::Node> {
            Some(&self.root)
        }

        fn children<'a>(&'a self, node: &'a Self::Node) -> impl Iterator<Item=&'a Self::Node> {
            self.table.get(node).into_iter().flatten()
        }
    }

    #[test]
    fn euler_only_root() {
        let tree = TestTree::new();

        let mut nodes = Vec::new();
        euler_traversal(&tree, &mut |x| nodes.push(*x));

        assert_eq!(vec![0], nodes);
    }

    #[test]
    fn euler_a() {
        let mut tree = TestTree::new();
        tree.link(0, vec![1]);

        let mut nodes = Vec::new();
        euler_traversal(&tree, &mut |x| nodes.push(*x));

        assert_eq!(vec![0, 1, 0], nodes);
    }

    #[test]
    fn euler_aa() {
        let mut tree = TestTree::new();
        tree.link(0, vec![1]);
        tree.link(1, vec![2]);

        let mut nodes = Vec::new();
        euler_traversal(&tree, &mut |x| nodes.push(*x));

        assert_eq!(vec![0, 1, 2, 1, 0], nodes);
    }

    #[test]
    fn euler_a_b() {
        let mut tree = TestTree::new();
        tree.link(0, vec![1, 2]);

        let mut nodes = Vec::new();
        euler_traversal(&tree, &mut |x| nodes.push(*x));

        assert_eq!(vec![0, 1, 0, 2, 0], nodes);
    }

    #[test]
    fn euler_aa_ab_ba_bb() {
        let mut tree = TestTree::new();
        tree.link(0, vec![1, 2]);
        tree.link(1, vec![3, 4]);
        tree.link(2, vec![5, 6]);

        let mut nodes = Vec::new();
        euler_traversal(&tree, &mut |x| nodes.push(*x));

        assert_eq!(vec![0, 1, 3, 1, 4, 1, 0, 2, 5, 2, 6, 2, 0], nodes);
    }
}