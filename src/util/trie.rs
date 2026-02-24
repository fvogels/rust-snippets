use std::fmt::Debug;
use std::{mem, usize};


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

    fn child(&mut self, parent_index: NodeId, child_index: usize) -> NodeId {
        self.grow_child_vector(parent_index, child_index);

        if self.nodes[parent_index].children[child_index] == usize::MAX {
            self.nodes[parent_index].children[child_index] = self.create_node(self.nodes[parent_index].depth + 1)
        }

        self.nodes[parent_index].children[child_index]
    }

    fn grow_child_vector(&mut self, parent_index: NodeId, child_index: usize) {
        while self.nodes[parent_index].children.len() <= child_index {
            self.nodes[parent_index].children.push(usize::MAX)
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

    fn link_nodes(&mut self) {
        let order = self.preorder_depth_first_order_node_traversal();
        let mut i = 0;
        let mut j = 0;

        loop {
            let mut min_upwards_depth: usize = usize::MAX;
            let mut max_downwards_depth: usize = self.nodes[order[j]].depth;

            while i == j || !self.nodes[order[j]].is_terminal() {
                let previous_node = &self.nodes[order[j]];

                j += 1;

                if j == order.len() {
                    return
                }

                let current_node = &self.nodes[order[j]];

                if previous_node.depth < current_node.depth {
                    // Going downwards
                    max_downwards_depth = std::cmp::max(max_downwards_depth, current_node.depth)
                }
                else {
                    // Going upwards
                    min_upwards_depth = std::cmp::min(min_upwards_depth, current_node.depth - 1)
                }
            }

            let link_depth = std::cmp::min(min_upwards_depth, max_downwards_depth);

            while i < j {
                self.nodes[order[i]].next_terminal = Some((link_depth, order[j]));
                i += 1
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

impl<T> Trie<T> {
    fn descend<'a>(&'a self, s: &str) -> Option<Cursor<'a, T>> {
        let mut current = 0;

        for b in s.as_bytes() {
            let ord = *b as usize;

            if ord < self.nodes[current].children.len() {
                current = self.nodes[current].children[ord]
            }
            else {
                return None
            }
        }

        if !self.nodes[current].is_terminal() {
            let (depth, next) = self.nodes[current].next_terminal?;

            if depth < s.len() {
                return None
            }

            current = next
        }

        Some(Cursor {
            parent: self,
            node_id: current,
            depth_cutoff: s.len(),
        })
    }

    pub fn iter<'a>(&'a self, s: &str) -> TrieIterator<'a, T> {
        TrieIterator::new(self.descend(s))
    }
}

#[cfg(test)]
mod test {
    // use rstest::rstest;
    use super::*;
    use pretty_assertions::{assert_eq};
    use anyhow::Result;
    use itertools::Itertools;
    use rstest::rstest;

    // use super::*;

    #[test]
    fn cursor_single_terminal() -> Result<()> {
        let mut builder = Builder::new();
        builder.add("a", 1);
        let trie = builder.finalize();

        let cursor = trie.descend("a").unwrap();
        let terminals = cursor.terminals();

        assert_eq!(terminals.len(), 1);
        assert_eq!(terminals[0], 1);
        Ok(())
    }

    #[test]
    fn cursor_two_terminals_in_same_node() -> Result<()> {
        let mut builder = Builder::new();
        builder.add("a", 1);
        builder.add("a", 2);
        let trie = builder.finalize();

        let mut cursor = trie.descend("a").unwrap();
        let terminals = cursor.terminals();

        assert_eq!(terminals.len(), 2);
        assert_eq!(terminals[0], 1);
        assert_eq!(terminals[1], 2);
        assert!(!cursor.next());
        Ok(())
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

            let mut cursor = trie.descend("a").unwrap();
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

            let mut cursor = trie.descend("b").unwrap();
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

        let mut cursor = trie.descend("a").unwrap();
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

            let mut cursor = trie.descend("a").unwrap();

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

            let mut cursor = trie.descend("aa").unwrap();

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
            let mut builder = Builder::new();
            let test_description = permutation.iter().map(|(x, _)| x).join("/");

            for (keyword, terminal) in permutation.into_iter() {
                builder.add(keyword, terminal)
            }
            let trie = builder.finalize();

            let mut iter = trie.iter("a");

            assert_eq!(iter.next(), Some(&vec![1]), "failed on permutation {}", test_description);
            assert_eq!(iter.next(), Some(&vec![2]), "failed on permutation {}", test_description);
            assert_eq!(iter.next(), None, "failed on permutation {}", test_description);
            println!("success for {}", test_description)
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