use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use crate::entities::segment::WeightedSegment;
use std::collections::btree_map::Entry;
use radix_heap::Radix;

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    cost: i64,
    position: usize,
}

impl Radix for State {
    #[inline]
    fn radix_similarity(&self, other: &State) -> u32 {
        (self.cost ^ other.cost).leading_zeros()
    }

    const RADIX_BITS: u32 = (std::mem::size_of::<i64>() * 8) as u32;
}

impl Ord for State {
    fn cmp(&self, other: &State) -> Ordering {
        self.cost.cmp(&other.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &State) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
struct Edge {
    cost: i64,
    node: usize,
}

pub struct Graph<'a> {
    ids: Vec<&'a str>,
    labels: BTreeMap<&'a str, usize>,
    adj_list: Vec<Vec<Edge>>
}

impl<'a> Graph<'a> {
    pub fn new(segments: Vec<WeightedSegment<'a>>) -> Graph<'a> {
        let labels = BTreeMap::new();
        let adj_list= vec!();
        let ids = vec!();
        let mut graph = Graph { ids, labels, adj_list };
        graph.add_edges(segments);
        graph
    }

    pub fn add_edges(&mut self, segments: Vec<WeightedSegment<'a>>) {
        for segment in segments {
            let from_label = self.get_label_mut(segment.segment.from);
            let to_label = self.get_label_mut(segment.segment.to);
            let edge = Edge {node: to_label, cost: segment.weight as i64};

            self.adj_list[from_label].push(edge);
        }
    }

    pub fn get_label_mut(&mut self, id: &'a str) -> usize {
        match self.labels.entry(id) {
            Entry::Vacant(v) => {
                let result = self.adj_list.len();
                v.insert(result);
                self.adj_list.push(Vec::new());
                self.ids.push(id);
                result
            }
            Entry::Occupied(ref label) => *label.get(),
        }
    }

    pub fn get_label(&self, id: &str) -> Option<&usize> {
        self.labels.get(id)
    }

    pub fn necessary_nodes(&self, from: &str, to: Vec<&str>) -> BTreeSet<String> {
        if self.get_label(from).is_none() {
            return BTreeSet::new();
        }
        let from_label = *self.get_label(from).unwrap();
        let mut to_labels: HashSet<usize> = to.iter()
            .map(|id| self.get_label(id))
            .filter(|id| id.is_some())
            .map(|id| *id.unwrap())
            .collect();
        let mut dist = vec![std::i64::MIN; self.adj_list.len()];
        let mut previous = vec![from_label; self.adj_list.len()];
        let mut heap = radix_heap::RadixHeapMap::new();

        dist[from_label] = 0;
        heap.push(0, from_label);

        while let Some((cost, position)) = heap.pop() {
            to_labels.remove(&position);

            if to_labels.is_empty() {
                break;
            }

            if cost < dist[position] {
                continue;
            }

            for edge in &self.adj_list[position] {
                let next_position = edge.node;
                let next_cost = cost - edge.cost;

                if next_cost > dist[next_position] {
                    heap.push(next_cost, next_position);
                    dist[next_position] = next_cost;
                    previous[next_position] = position;
                }
            }
        }

        let mut result = BTreeSet::new();

        for to_id in to {
            if self.get_label(to_id).is_none() {
                continue;
            }
            let mut current_label = *self.get_label(to_id).unwrap();
            result.insert(self.ids[current_label].to_owned());
            while previous[current_label] != from_label {
                result.insert(self.ids[current_label].to_owned());
                current_label = previous[current_label];
            }
            result.insert(self.ids[current_label].to_owned());
        }

        return result
    }
}