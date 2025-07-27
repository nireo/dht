use std::time::Instant;

use indexmap::IndexMap;

use crate::node::{Node, NodeId};

#[derive(Debug)]
struct KBucket {
    rend: u128,
    rstart: u128,
    nodes: IndexMap<NodeId, Node>,
    replacement_nodes: IndexMap<NodeId, Node>,
    last_updated: Instant,
    ksize: usize,
    max_replacement_nodes: usize,
}

impl KBucket {
    pub fn new(
        range_low: u128,
        range_upper: u128,
        ksize: usize,
        replacement_node_factor: usize,
    ) -> Self {
        Self {
            rstart: range_low,
            rend: range_upper,
            nodes: IndexMap::new(),
            replacement_nodes: IndexMap::new(),
            last_updated: Instant::now(),
            ksize,
            max_replacement_nodes: ksize * replacement_node_factor,
        }
    }

    pub fn update_ts(&mut self) {
        self.last_updated = Instant::now();
    }

    pub fn get_nodes(&self) -> Vec<Node> {
        self.nodes.values().cloned().collect()
    }

    pub fn split(&self) -> (KBucket, KBucket) {
        let midpoint = (self.rstart + self.rend) / 2;
        let mut one = KBucket::new(
            self.rstart,
            midpoint,
            self.ksize,
            self.max_replacement_nodes / self.ksize,
        );
        let mut two = KBucket::new(
            midpoint + 1,
            self.rend,
            self.ksize,
            self.max_replacement_nodes / self.ksize,
        );

        let all_nodes = self.nodes.values().chain(self.replacement_nodes.values());
        for node in all_nodes {
            let node_id_as_u128 = node_id_to_u128(&node.id);
            if node_id_as_u128 <= midpoint {
                one.add_node(node.clone());
            } else {
                two.add_node(node.clone());
            }
        }

        (one, two)
    }

    pub fn remove_node(&mut self, node: &Node) {
        self.replacement_nodes.shift_remove(&node.id);

        if self.nodes.shift_remove(&node.id).is_some() {
            if let Some((new_node_id, new_node)) = self.replacement_nodes.shift_remove_index(0) {
                self.nodes.insert(new_node_id, new_node);
            }
        }
    }

    pub fn has_in_range(&self, node: &Node) -> bool {
        let idc = node_id_to_u128(&node.id);
        self.rstart <= idc && idc <= self.rend
    }

    pub fn is_new_node(&self, node: &Node) -> bool {
        !self.nodes.contains_key(&node.id)
    }

    /// Add a node to the bucket
    ///
    /// Returns `true` if the node was added to the main bucket,
    /// `false` if it was added to replacement nodes or bucket is full
    pub fn add_node(&mut self, node: Node) -> bool {
        let node_id = node.id;

        if self.nodes.contains_key(&node_id) {
            self.nodes.shift_remove(&node_id);
            self.nodes.insert(node_id, node);
            return true;
        }

        if self.nodes.len() < self.ksize {
            self.nodes.insert(node_id, node);
            return true;
        }

        self.replacement_nodes.shift_remove(&node_id);
        self.replacement_nodes.insert(node_id, node);

        while self.replacement_nodes.len() > self.max_replacement_nodes {
            self.replacement_nodes.shift_remove_index(0);
        }
        false
    }

    pub fn depth(&self) -> usize {
        if self.nodes.is_empty() {
            return 0;
        }

        let bit_strings: Vec<String> = self
            .nodes
            .values()
            .map(|node| node_id_to_bit_string(&node.id))
            .collect();

        shared_prefix(&bit_strings).len()
    }

    pub fn head(&self) -> Option<&Node> {
        self.nodes.values().next()
    }

    pub fn get(&self, node_id: &NodeId) -> Option<&Node> {
        self.nodes.get(node_id)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.nodes.len() >= self.ksize
    }

    pub fn range(&self) -> (u128, u128) {
        (self.rstart, self.rend)
    }

    pub fn last_updated(&self) -> Instant {
        self.last_updated
    }

    pub fn replacement_count(&self) -> usize {
        self.replacement_nodes.len()
    }

    pub fn get_replacement_nodes(&self) -> Vec<Node> {
        self.replacement_nodes.values().cloned().collect()
    }
}

fn node_id_to_u128(node_id: &NodeId) -> u128 {
    let bytes = node_id.as_bytes();
    let mut u128_bytes = [0u8; 16];
    u128_bytes.copy_from_slice(&bytes[0..16]);
    u128::from_be_bytes(u128_bytes)
}

fn node_id_to_bit_string(node_id: &NodeId) -> String {
    let bytes = node_id.as_bytes();
    bytes
        .iter()
        .map(|byte| format!("{:08b}", byte))
        .collect::<String>()
}

fn shared_prefix(bit_strings: &[String]) -> String {
    if bit_strings.is_empty() {
        return String::new();
    }

    let first = &bit_strings[0];
    let mut prefix = String::new();

    for (i, char) in first.chars().enumerate() {
        if bit_strings.iter().all(|s| s.chars().nth(i) == Some(char)) {
            prefix.push(char);
        } else {
            break;
        }
    }

    prefix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kbucket_creation() {
        let bucket = KBucket::new(0, 100, 20, 5);
        assert_eq!(bucket.len(), 0);
        assert!(bucket.is_empty());
        assert!(!bucket.is_full());
        assert_eq!(bucket.range(), (0, 100));
    }

    #[test]
    fn test_add_node() {
        let mut bucket = KBucket::new(0, u128::MAX, 2, 5);
        let node1 = Node::new(NodeId::random());
        let node2 = Node::new(NodeId::random());
        let node3 = Node::new(NodeId::random());

        assert!(bucket.add_node(node1.clone()));
        assert!(bucket.add_node(node2.clone()));
        assert_eq!(bucket.len(), 2);
        assert!(bucket.is_full());

        assert!(!bucket.add_node(node3.clone()));
        assert_eq!(bucket.len(), 2);
        assert_eq!(bucket.replacement_count(), 1);
    }

    #[test]
    fn test_remove_node() {
        let mut bucket = KBucket::new(0, u128::MAX, 2, 5);
        let node1 = Node::new(NodeId::random());
        let node2 = Node::new(NodeId::random());
        let node3 = Node::new(NodeId::random());

        bucket.add_node(node1.clone());
        bucket.add_node(node2.clone());
        bucket.add_node(node3.clone());

        bucket.remove_node(&node1);
        assert_eq!(bucket.len(), 2);
        assert_eq!(bucket.replacement_count(), 0);
        assert!(bucket.get(&node3.id).is_some());
    }

    #[test]
    fn test_split() {
        let mut bucket = KBucket::new(0, 200, 20, 5);

        let node1 = Node::new(NodeId::from_slice(&[0u8; 20]).unwrap());
        bucket.add_node(node1);

        let (left, right) = bucket.split();
        assert_eq!(left.range().1, 100);
        assert_eq!(right.range().0, 101);
    }
}
