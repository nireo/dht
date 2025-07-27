use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashSet},
};

use crate::node::{Node, NodeId};

#[derive(Debug, Clone)]
struct HeapEntry {
    distance: NodeId,
    node: Node,
}

impl HeapEntry {
    fn new(distance: NodeId, node: Node) -> Self {
        Self { distance, node }
    }
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Eq for HeapEntry {}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // reverse the ordering so BinaryHeap becomes a min-heap
        other.distance.cmp(&self.distance)
    }
}

// NodeHeap is a heap of nodes ordered by distance to a given node.
pub struct NodeHeap {
    node: Node,
    heap: BinaryHeap<HeapEntry>,
    contacted: HashSet<NodeId>,
    max_size: usize,
}

impl NodeHeap {
    pub fn new(node: Node, max_size: usize) -> Self {
        Self {
            node,
            heap: BinaryHeap::new(),
            contacted: HashSet::new(),
            max_size,
        }
    }

    /// Note that while this heap retains a constant visible size (based on the iterator),
    /// its actual size may be quite a bit larger than what's exposed. Therefore,
    /// removal of nodes may not change the visible size as previously added
    /// nodes suddenly become visible.
    pub fn remove<I>(&mut self, peers: I)
    where
        I: IntoIterator<Item = NodeId>,
    {
        let peers: HashSet<NodeId> = peers.into_iter().collect();
        if peers.is_empty() {
            return;
        }

        // Rebuild the heap without the removed peers
        let old_heap = std::mem::take(&mut self.heap);
        self.heap = old_heap
            .into_iter()
            .filter(|entry| !peers.contains(&entry.node.id))
            .collect();
    }

    pub fn get_node(&self, node_id: &NodeId) -> Option<&Node> {
        self.heap
            .iter()
            .find(|entry| entry.node.id == *node_id)
            .map(|entry| &entry.node)
    }

    pub fn have_contacted_all(&self) -> bool {
        self.get_uncontacted().is_empty()
    }

    pub fn get_ids(&self) -> Vec<NodeId> {
        self.iter().map(|node| node.id).collect()
    }

    pub fn mark_contacted(&mut self, node: &Node) {
        self.contacted.insert(node.id);
    }

    /// Pop the closest node from the heap
    pub fn pop_left(&mut self) -> Option<Node> {
        self.heap.pop().map(|entry| entry.node)
    }

    pub fn push<I>(&mut self, nodes: I)
    where
        I: IntoIterator<Item = Node>,
    {
        for node in nodes {
            if !self.contains(&node) {
                let distance = self.node.distance_to(&node);
                let entry = HeapEntry::new(distance, node);
                self.heap.push(entry);
            }
        }
    }

    pub fn push_one(&mut self, node: Node) {
        self.push(std::iter::once(node));
    }

    pub fn len(&self) -> usize {
        std::cmp::min(self.heap.len(), self.max_size)
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn contains(&self, node: &Node) -> bool {
        self.heap.iter().any(|entry| entry.node.id == node.id)
    }

    pub fn get_uncontacted(&self) -> Vec<Node> {
        self.iter()
            .filter(|node| !self.contacted.contains(&node.id))
            .cloned()
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Node> {
        let mut entries: Vec<_> = self.heap.iter().collect();
        entries.sort_by(|a, b| a.distance.cmp(&b.distance));
        entries
            .into_iter()
            .take(self.max_size)
            .map(|entry| &entry.node)
    }

    pub fn to_vec(&self) -> Vec<Node> {
        self.iter().cloned().collect()
    }

    pub fn actual_size(&self) -> usize {
        self.heap.len()
    }

    pub fn clear(&mut self) {
        self.heap.clear();
        self.contacted.clear();
    }

    pub fn reference_node(&self) -> &Node {
        &self.node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_heap_creation() {
        let reference_node = Node::new(NodeId::random());
        let heap = NodeHeap::new(reference_node.clone(), 20);

        assert_eq!(heap.len(), 0);
        assert!(heap.is_empty());
        assert_eq!(heap.reference_node().id, reference_node.id);
    }

    #[test]
    fn test_push_and_contains() {
        let reference_node = Node::new(NodeId::random());
        let mut heap = NodeHeap::new(reference_node, 20);

        let test_node = Node::new(NodeId::random());
        heap.push_one(test_node.clone());

        assert_eq!(heap.len(), 1);
        assert!(heap.contains(&test_node));
    }

    #[test]
    fn test_mark_contacted() {
        let reference_node = Node::new(NodeId::random());
        let mut heap = NodeHeap::new(reference_node, 20);

        let test_node = Node::new(NodeId::random());
        heap.push_one(test_node.clone());
        heap.mark_contacted(&test_node);

        assert!(heap.have_contacted_all());
        assert!(heap.get_uncontacted().is_empty());
    }

    #[test]
    fn test_maxsize_limiting() {
        let reference_node = Node::new(NodeId::random());
        let mut heap = NodeHeap::new(reference_node, 2);

        for _ in 0..3 {
            heap.push_one(Node::new(NodeId::random()));
        }

        assert_eq!(heap.len(), 2);
        assert_eq!(heap.actual_size(), 3);
    }
}
