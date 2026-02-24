use std::fmt::Debug;
use std::{mem, usize};

use crate::util::tree_traversal;


type NodeId = usize;

pub struct Trie<T> {
    nodes: Vec<Node<T>>,
}

pub struct Builder<T>{
    nodes: Vec<Node<T>>,
}

struct Node<T> {
    children: Vec<NodeId>,
    terminals: Vec<T>,
    next_terminal: Option<(usize, NodeId)>,
    depth: usize,
}

impl<T> Node<T> {
    fn is_terminal(&self) -> bool {
        !self.terminals.is_empty()
    }
}

struct Cursor<'a, T> {
    parent: &'a Trie<T>,
    node_id: NodeId,
    depth_cutoff: usize,
}

impl<'a, T> Cursor<'a, T> {
    pub fn next(&mut self) -> bool {
        // println!("moving from {}", self.node_id);

        match self.parent.nodes[self.node_id].next_terminal {
            None => {
                false
            },
            Some((depth, next)) => {
                if depth < self.depth_cutoff {
                    false
                }
                else {
                    self.node_id = next;
                    // println!("moved to {}", id);
                    true
                }
            }
        }
    }

    pub fn terminals(&self) -> &'a Vec<T> {
        &self.parent.nodes[self.node_id].terminals
    }
}

pub struct TrieIterator<'a, T> {
    data: Option<(&'a Vec<T>, Cursor<'a, T>)>,
}

impl<'a, T> TrieIterator<'a, T> {
    fn new(cursor: Option<Cursor<'a, T>>) -> TrieIterator<'a, T> {
        match cursor {
            Some(c) => {
                let next = c.terminals();

                TrieIterator{
                    data: Some((next, c)),
                }
            },
            None => {
                TrieIterator{
                    data: None,
                }
            }
        }
    }
}

impl<'a, T> Iterator for TrieIterator<'a, T> {
    type Item = &'a Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let data = mem::replace(&mut self.data, None);
        let (current, mut cursor) = data?;

        if cursor.next() {
            self.data = Some((cursor.terminals(), cursor))
        }

        Some(current)
    }
}

impl<T> Builder<T> where T: Debug {
    pub fn new() -> Builder<T> {
        let root: Node<T> = Node{ children: Vec::new(), terminals: Vec::new(), next_terminal: None, depth: 0 };

        Builder{
            nodes: vec![ root ],
        }
    }

    pub fn add(&mut self, keyword: &str, terminal: T) {
        let mut current: NodeId = 0;

        for b in keyword.bytes() {
            let ord = b as usize;

            current = self.child(current, ord);
        }

        self.nodes[current].terminals.push(terminal)
    }

    fn child(&mut self, parent_id: NodeId, ord: usize) -> NodeId {
        self.grow_child_vector(parent_id, ord);

        if self.nodes[parent_id].children[ord] == usize::MAX {
            self.nodes[parent_id].children[ord] = self.create_node(self.nodes[parent_id].depth + 1)
        }

        self.nodes[parent_id].children[ord]
    }

    fn grow_child_vector(&mut self, parent_id: NodeId, child_index: usize) {
        while self.nodes[parent_id].children.len() <= child_index {
            self.nodes[parent_id].children.push(usize::MAX)
        }
    }

    fn create_node(&mut self, depth: usize) -> NodeId {
        let node_id = self.nodes.len();
        let node = Node{
            children: Vec::new(),
            terminals: Vec::new(),
            next_terminal: None,
            depth: depth,
        };

        self.nodes.push(node);
        node_id
    }

    fn preorder_depth_first_order_node_traversal(&self) -> Vec<NodeId> {
        let mut queue = vec![0];
        let mut result = Vec::new();

        loop {
            match queue.pop() {
                Some(next) => {
                    result.push(next);

                    for j in self.nodes[next].children.iter().copied().rev() {
                        if j != usize::MAX {
                            queue.push(j)
                        }
                    }
                },
                None => {
                    return result
                },
            }
        }
    }

    fn euler_node_traversal(&self) -> Vec<NodeId> {
        let mut result = Vec::new();

        tree_traversal::euler_traversal(self,  &mut |n| result.push(*n));

        result
    }

    fn link_nodes(&mut self) {
        let link_order: Vec<NodeId> = self.preorder_depth_first_order_node_traversal();
        let euler: Vec<NodeId> = self.euler_node_traversal();
        debug_assert!(!link_order.is_empty());

        let mut link_order_index = link_order.len() - 1;
        debug_assert!(self.nodes[link_order_index].is_terminal());

        let mut euler_index = euler.len() - 1;
        while euler[euler_index] != link_order[link_order_index] {
            debug_assert!(euler_index > 0);
            euler_index -= 1
        }

        let mut terminal_id = link_order[link_order_index];
        let mut depth = self.nodes[terminal_id].depth;
        while link_order_index > 0 {
            let target = link_order[link_order_index-1];

            while euler[euler_index] != target {
                let from_node = euler[euler_index];
                euler_index -= 1;
                let to_node = euler[euler_index];

                if self.nodes[from_node].depth < self.nodes[to_node].depth {
                    depth = std::cmp::min(depth, self.nodes[from_node].depth)
                }

                self.nodes[to_node].next_terminal = Some((depth, terminal_id));
            }

            link_order_index -= 1;
            let node = &self.nodes[link_order[link_order_index]];
            if node.is_terminal() {
                terminal_id = link_order[link_order_index];
                depth = node.depth;
            }
        }
    }

    pub fn finalize(mut self) -> Trie<T> {
        self.link_nodes();

        return Trie{
            nodes: self.nodes,
        }
    }
}

impl<T> tree_traversal::Tree for Builder<T> {
    type Node = NodeId;

    fn root(&self) -> Option<&Self::Node> {
        Some(&0)
    }

    fn children<'a>(&'a self, node: &NodeId) -> impl Iterator<Item=&'a Self::Node> {
        self.nodes[*node].children.iter().filter(|x| **x != usize::MAX)
    }
}

impl<T> Trie<T> {
    fn descend<'a>(&'a self, key: &str) -> Option<NodeId> {
        let mut current: NodeId = 0;

        for c in key.as_bytes() {
            let ord = *c as usize;

            current = self.child(current, ord)?;
        }

        Some(current)
    }

    fn child<'a>(&'a self, parent_id: NodeId, child_index: usize) -> Option<NodeId> {
        let children = &self.nodes[parent_id].children;

        if child_index < children.len() && children[child_index] != usize::MAX {
            Some(children[child_index])
        }
        else {
            None
        }
    }

    fn cursor<'a>(&'a self, s: &str) -> Option<Cursor<'a, T>> {
        let mut node_id = self.descend(s)?;

        if !self.nodes[node_id].is_terminal() {
            let (depth, next) = self.nodes[node_id].next_terminal?;

            if depth < s.len() {
                return None
            }

            node_id = next
        }

        Some(Cursor {
            parent: self,
            node_id: node_id,
            depth_cutoff: s.len(),
        })
    }

    #[cfg(test)]
    fn node<'a>(&'a self, node_id: NodeId) -> Option<&'a Node<T>> {
        self.nodes.get(node_id)
    }

    #[cfg(test)]
    fn descend_to_node<'a>(&'a self, key: &str) -> Option<&'a Node<T>> {
        let node_id = self.descend(key)?;
        self.node(node_id)
    }

    pub fn iter<'a>(&'a self, s: &str) -> TrieIterator<'a, T> {
        TrieIterator::new(self.cursor(s))
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;
    use super::*;
    use pretty_assertions::{assert_eq};
    use anyhow::Result;
    use itertools::Itertools;


    #[test]
    fn is_terminal_a() {
        let mut builder = Builder::new();
        builder.add("a", 1);
        let tree = builder.finalize();

        let root = tree.descend_to_node("").unwrap();
        let child_a = tree.descend_to_node("a").unwrap();

        assert!(!root.is_terminal());
        assert!(child_a.is_terminal());
    }

    #[test]
    fn is_terminal_aa() {
        let mut builder = Builder::new();
        builder.add("aa", 1);
        let tree = builder.finalize();

        let root = tree.descend_to_node("").unwrap();
        let child_a = tree.descend_to_node("a").unwrap();
        let child_aa = tree.descend_to_node("aa").unwrap();

        assert!(!root.is_terminal());
        assert!(!child_a.is_terminal());
        assert!(child_aa.is_terminal());
    }

    #[test]
    fn linking_a() {
        let mut builder = Builder::new();
        builder.add("a", 1);
        let tree = builder.finalize();

        let root = tree.descend_to_node("").unwrap();
        let node_id_a = tree.descend("a").unwrap();
        let node_a = tree.node(node_id_a).unwrap();

        assert_eq!(root.next_terminal, Some((1, node_id_a)));
        assert_eq!(node_a.next_terminal, None)
    }

    #[test]
    fn linking_aa() {
        let mut builder = Builder::new();
        builder.add("aa", 1);
        let tree = builder.finalize();

        let root = tree.descend_to_node("").unwrap();
        let node_id_aa = tree.descend("aa").unwrap();
        let node_aa = tree.node(node_id_aa).unwrap();

        assert_eq!(root.next_terminal, Some((2, node_id_aa)));
        assert_eq!(node_aa.next_terminal, None)
    }

    #[test]
    fn linking_a_b() {
        let mut builder = Builder::new();
        builder.add("a", 1);
        builder.add("b", 2);
        let tree = builder.finalize();

        let root = tree.descend_to_node("").unwrap();
        let node_id_a = tree.descend("a").unwrap();
        let node_id_b = tree.descend("b").unwrap();
        let node_a = tree.node(node_id_a).unwrap();
        let node_b = tree.node(node_id_b).unwrap();

        assert_eq!(root.next_terminal, Some((1, node_id_a)));
        assert_eq!(node_a.next_terminal, Some((0, node_id_b)));
        assert_eq!(node_b.next_terminal, None);
    }

    #[test]
    fn linking_a_bb() {
        let mut builder = Builder::new();
        builder.add("a", 1);
        builder.add("bb", 2);
        let tree = builder.finalize();

        let root = tree.descend_to_node("").unwrap();
        let node_id_a = tree.descend("a").unwrap();
        let node_id_b = tree.descend("b").unwrap();
        let node_id_bb = tree.descend("bb").unwrap();
        let node_a = tree.node(node_id_a).unwrap();
        let node_b = tree.node(node_id_b).unwrap();
        let node_bb = tree.node(node_id_bb).unwrap();

        assert_eq!(root.next_terminal, Some((1, node_id_a)));
        assert_eq!(node_a.next_terminal, Some((0, node_id_bb)));
        assert_eq!(node_b.next_terminal, Some((2, node_id_bb)));
        assert_eq!(node_bb.next_terminal, None);
    }

    #[test]
    fn cursor_single_terminal() {
        let mut builder = Builder::new();
        builder.add("a", 1);
        let trie = builder.finalize();

        let cursor = trie.cursor("a").unwrap();
        let terminals = cursor.terminals();

        assert_eq!(terminals.len(), 1);
        assert_eq!(terminals[0], 1)
    }

    #[test]
    fn cursor_two_terminals_in_same_node(){
        let mut builder = Builder::new();
        builder.add("a", 1);
        builder.add("a", 2);
        let trie = builder.finalize();

        let mut cursor = trie.cursor("a").unwrap();
        let terminals = cursor.terminals();

        assert_eq!(terminals.len(), 2);
        assert_eq!(terminals[0], 1);
        assert_eq!(terminals[1], 2);
        assert!(!cursor.next())
    }

    #[test]
    fn cursor_two_terminals_in_separate_nodes_a_b_descend_a() {
        let pairs = vec![("a", 1), ("b", 2)];
        let size = pairs.len();

        for permutation in pairs.into_iter().permutations(size) {
            let mut builder = Builder::new();

            for (keyword, terminal) in permutation.into_iter() {
                builder.add(keyword, terminal)
            }
            let trie = builder.finalize();

            let mut cursor = trie.cursor("a").unwrap();
            let terminals = cursor.terminals();

            assert_eq!(terminals.len(), 1);
            assert_eq!(terminals[0], 1);
            assert!(!cursor.next());
        }
    }

    #[test]
    fn cursor_two_terminals_in_separate_nodes_descend_b() {
        let pairs = vec![("a", 1), ("b", 2)];
        let size = pairs.len();

        for permutation in pairs.into_iter().permutations(size) {
            let mut builder = Builder::new();

            for (keyword, terminal) in permutation.into_iter() {
                builder.add(keyword, terminal)
            }
            let trie = builder.finalize();

            let mut cursor = trie.cursor("b").unwrap();
            let terminals = cursor.terminals();

            assert_eq!(terminals.len(), 1);
            assert_eq!(terminals[0], 2);
            assert!(!cursor.next());
        }
    }

    #[test]
    fn cursor_descend_to_nonterminal() -> Result<()> {
        let mut builder = Builder::new();
        builder.add("aa", 1);
        let trie = builder.finalize();

        let mut cursor = trie.cursor("a").unwrap();
        let terminals = cursor.terminals();

        assert_eq!(terminals.len(), 1);
        assert_eq!(terminals[0], 1);
        assert!(!cursor.next());
        Ok(())
    }

    #[test]
    fn cursor_next_three_terminals_descend_a() {
        let pairs = vec![("aa", 1), ("aab", 2), ("aac", 3)];
        let size = pairs.len();

        for permutation in pairs.into_iter().permutations(size) {
            let test_description = permutation.iter().map(|(x, _)| x).join("/");
            println!("Permutation {}", test_description);

            let mut builder = Builder::new();

            for (keyword, terminal) in permutation.into_iter() {
                builder.add(keyword, terminal)
            }
            let trie = builder.finalize();

            let mut cursor = trie.cursor("a").unwrap();

            assert_eq!(cursor.terminals().len(), 1);
            assert_eq!(cursor.terminals()[0], 1);
            assert!(cursor.next());
            assert_eq!(cursor.terminals().len(), 1);
            assert_eq!(cursor.terminals()[0], 2);
            assert!(cursor.next());
            assert_eq!(cursor.terminals().len(), 1);
            assert_eq!(cursor.terminals()[0], 3);
            assert!(!cursor.next());
        }
    }

    #[test]
    fn cursor_next() {
        let pairs = vec![("aa", 1), ("aab", 2), ("aac", 3)];
        let size = pairs.len();

        for permutation in pairs.into_iter().permutations(size) {
            let test_description = permutation.iter().map(|(x, _)| x).join("/");
            println!("{}", test_description);

            let mut builder = Builder::new();
            for (keyword, terminal) in permutation.into_iter() {
                builder.add(keyword, terminal)
            }
            let trie = builder.finalize();

            let mut cursor = trie.cursor("aa").unwrap();

            assert_eq!(cursor.terminals().len(), 1);
            assert_eq!(cursor.terminals()[0], 1);
            assert!(cursor.next());
            assert_eq!(cursor.terminals().len(), 1);
            assert_eq!(cursor.terminals()[0], 2);
            assert!(cursor.next());
            assert_eq!(cursor.terminals().len(), 1);
            assert_eq!(cursor.terminals()[0], 3);
            assert!(!cursor.next());
        }
    }

    #[test]
    fn iterating_aa_aab_at_a() {
        let pairs = vec![("aa", 1), ("aab", 2)];
        let size = pairs.len();

        for permutation in pairs.into_iter().permutations(size) {
            let test_description = permutation.iter().map(|(x, _)| x).join("/");
            println!("{}", test_description);

            let mut builder = Builder::new();
            for (keyword, terminal) in permutation.into_iter() {
                builder.add(keyword, terminal)
            }
            let trie = builder.finalize();

            let mut iter = trie.iter("a");

            assert_eq!(iter.next(), Some(&vec![1]));
            assert_eq!(iter.next(), Some(&vec![2]));
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn iterating_aa_aab_at_b() {
        let pairs = vec![("aa", 1), ("aab", 2)];
        let size = pairs.len();

        for permutation in pairs.into_iter().permutations(size) {
            let mut builder = Builder::new();
            let test_description = permutation.iter().map(|(x, _)| x).join("/");

            for (keyword, terminal) in permutation.into_iter() {
                builder.add(keyword, terminal)
            }
            let trie = builder.finalize();

            let mut iter = trie.iter("b");

            assert_eq!(iter.next(), None, "failed on permutation {}", test_description);
        }
    }

    #[test]
    fn iterating_aa_aab_aac_at_a() {
        let pairs = vec![("aa", 1), ("aab", 2), ("aac", 3)];
        let size = pairs.len();

        for permutation in pairs.into_iter().permutations(size) {
            let mut builder = Builder::new();
            let test_description = permutation.iter().map(|(x, _)| x).join("/");

            for (keyword, terminal) in permutation.into_iter() {
                builder.add(keyword, terminal)
            }
            let trie = builder.finalize();

            let mut iter = trie.iter("a");

            assert_eq!(iter.next(), Some(&vec![1]), "failed on permutation {}", test_description);
            assert_eq!(iter.next(), Some(&vec![2]), "failed on permutation {}", test_description);
            assert_eq!(iter.next(), Some(&vec![3]), "failed on permutation {}", test_description);
            assert_eq!(iter.next(), None, "failed on permutation {}", test_description);
        }
    }

    #[test]
    fn iterating_aa_aab_aac_at_b() {
        let pairs = vec![("aa", 1), ("aab", 2), ("aac", 3)];
        let size = pairs.len();

        for permutation in pairs.into_iter().permutations(size) {
            let mut builder = Builder::new();
            let test_description = permutation.iter().map(|(x, _)| x).join("/");

            for (keyword, terminal) in permutation.into_iter() {
                builder.add(keyword, terminal)
            }
            let trie = builder.finalize();

            let mut iter = trie.iter("b");

            assert_eq!(iter.next(), None, "failed on permutation {}", test_description);
        }
    }

    #[test]
    fn iterating_aa_ab_c_at_a() {
        let pairs = vec![("aa", 1), ("ab", 2), ("b", 3)];
        let size = pairs.len();

        for permutation in pairs.into_iter().permutations(size) {
            let test_description = permutation.iter().map(|(x, _)| x).join("/");
            println!("{}", test_description);

            let mut builder = Builder::new();

            for (keyword, terminal) in permutation.into_iter() {
                builder.add(keyword, terminal)
            }
            let trie = builder.finalize();

            let mut iter = trie.iter("a");

            assert_eq!(iter.next(), Some(&vec![1]), "failed on permutation {}", test_description);
            assert_eq!(iter.next(), Some(&vec![2]), "failed on permutation {}", test_description);
            assert_eq!(iter.next(), None, "failed on permutation {}", test_description);
        }
    }

    #[test]
    fn iterating_4() {
        let pairs = vec![("aa", 1), ("aab", 2), ("aac", 3), ("ab", 4)];
        let size = pairs.len();

        for permutation in pairs.into_iter().permutations(size) {
            let test_description = permutation.iter().map(|(x, _)| x).join("/");
            println!("{}", test_description);

            let mut builder = Builder::new();

            for (keyword, terminal) in permutation.into_iter() {
                builder.add(keyword, terminal)
            }
            let trie = builder.finalize();

            let mut iter = trie.iter("a");

            assert_eq!(iter.next(), Some(&vec![1]));
            assert_eq!(iter.next(), Some(&vec![2]));
            assert_eq!(iter.next(), Some(&vec![3]));
            assert_eq!(iter.next(), Some(&vec![4]));
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn iterating_with_cutoff() -> Result<()> {
        let mut builder = Builder::new();
        builder.add("aa", 1);
        builder.add("aab", 2);
        builder.add("aac", 3);
        builder.add("bb", 4);
        let trie = builder.finalize();

        let mut iter = trie.iter("aa");

        assert_eq!(iter.next(), Some(&vec![1]));
        assert_eq!(iter.next(), Some(&vec![2]));
        assert_eq!(iter.next(), Some(&vec![3]));
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[test]
    fn iterating_same_terminal_2() {
        let mut builder = Builder::new();
        builder.add("ab", 1);
        builder.add("xy", 1);
        let trie = builder.finalize();

        let mut iter = trie.iter("a");

        assert_eq!(iter.next(), Some(&vec![1]));
        assert_eq!(iter.next(), None)
    }

    #[test]
    fn iterating_same_terminal_3() -> Result<()> {
        let mut builder = Builder::new();
        builder.add("a", 1);
        builder.add("f", 1);
        builder.add("re", 1);
        let trie = builder.finalize();

        let mut iter = trie.iter("r");

        assert_eq!(iter.next(), Some(&vec![1]));
        assert_eq!(iter.next(), None);
        Ok(())
    }

    #[rstest]
    #[case("r")]
    #[case("re")]
    #[case("rem")]
    #[case("remo")]
    #[case("remov")]
    #[case("remove")]
    fn iterating_same_terminal_4(#[case] keyword: &str) -> Result<()> {
        let mut builder = Builder::new();
        builder.add("remove", 1);
        builder.add("accents", 1);
        builder.add("from", 1);
        builder.add("strings", 1);
        let trie = builder.finalize();

        let mut iter = trie.iter(keyword);

        assert_eq!(iter.next(), Some(&vec![1]));
        assert_eq!(iter.next(), None);
        Ok(())
    }
}