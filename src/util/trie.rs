use std::marker::PhantomData;
use std::fmt::Debug;
use std::mem;


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
    next_terminal: Option<NodeId>,
    depth: usize,
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
            Some(id) => {
                if self.parent.nodes[id].depth <= self.depth_cutoff {
                    false
                }
                else {
                    self.node_id = id;
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
        let mut depth = 1;

        for b in keyword.bytes() {
            let ord = b as usize;
            while self.nodes[current].children.len() <= ord {
                let node_id = self.create_node(depth);
                self.nodes[current].children.push(node_id)
            }

            current = self.nodes[current].children[ord];
            depth += 1
        }

        // println!("Node {} gets terminal {:?}", current, terminal);
        self.nodes[current].terminals.push(terminal)
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

    fn depth_first_order_node_traversal(&self) -> Vec<NodeId> {
        let mut queue = vec![0];
        let mut result = Vec::new();

        loop {
            match queue.pop() {
                Some(next) => {
                    result.push(next);

                    for j in self.nodes[next].children.iter().rev() {
                        queue.push(*j)
                    }
                },
                None => {
                    return result
                },
            }


        }
    }

    fn link_nodes(&mut self) {
        let order = self.depth_first_order_node_traversal();
        let mut i = 0;
        let mut j = 1;

        while j < order.len() {
            while j < order.len() && self.nodes[order[j]].terminals.is_empty() {
                j += 1
            }
            if j < order.len() {
                while i < j {
                    // println!("{} -> {}", i, j);
                    self.nodes[order[i]].next_terminal = Some(j);
                    i += 1
                }
            }
            j += 1;
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

            if ord < self.nodes.len() {
                current = self.nodes[current].children[ord]
            }
            else {
                return None
            }
        }

        if self.nodes[current].terminals.is_empty() {
            current = self.nodes[current].next_terminal?;
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

    // use super::*;

    #[test]
    fn test_single_terminal() -> Result<()> {
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
    fn test_double_terminal() -> Result<()> {
        let mut builder = Builder::new();
        builder.add("a", 1);
        builder.add("a", 2);
        let trie = builder.finalize();

        let cursor = trie.descend("a").unwrap();
        let terminals = cursor.terminals();

        assert_eq!(terminals.len(), 2);
        assert_eq!(terminals[0], 1);
        assert_eq!(terminals[1], 2);
        Ok(())
    }

    #[test]
    fn test_descend_to_nonterminal() -> Result<()> {
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
    fn test_next() -> Result<()> {
        let mut builder = Builder::new();
        builder.add("aa", 1);
        builder.add("aab", 2);
        builder.add("aac", 3);
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
        Ok(())
    }

    #[test]
    fn test_next2() -> Result<()> {
        let mut builder = Builder::new();
        builder.add("aa", 1);
        builder.add("aab", 2);
        builder.add("aac", 3);
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
        Ok(())
    }

    #[test]
    fn iterating() -> Result<()> {
        let mut builder = Builder::new();
        builder.add("aa", 1);
        builder.add("aab", 2);
        builder.add("aac", 3);
        builder.add("bb", 4);
        let trie = builder.finalize();

        let mut iter = trie.iter("a");

        assert_eq!(iter.next(), Some(&vec![1]));
        assert_eq!(iter.next(), Some(&vec![2]));
        assert_eq!(iter.next(), Some(&vec![3]));
        assert_eq!(iter.next(), Some(&vec![4]));
        assert_eq!(iter.next(), None);
        Ok(())
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
}