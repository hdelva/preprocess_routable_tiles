use crate::entities::segment::WeightedSegment;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet, HashSet};

#[derive(Debug)]
pub struct Edge {
    pub cost: i64,
    pub node: usize,
}

pub struct Graph<'a> {
    pub ids: Vec<&'a str>,
    pub labels: BTreeMap<&'a str, usize>,
    pub adj_list: Vec<Vec<Edge>>,
}

impl<'a> Graph<'a> {
    pub fn new(segments: Vec<WeightedSegment<'a>>) -> Graph<'a> {
        let labels = BTreeMap::new();
        let adj_list = vec![];
        let ids = vec![];
        let mut graph = Graph {
            ids,
            labels,
            adj_list,
        };
        graph.add_edges(segments);
        graph
    }

    pub fn add_edges(&mut self, segments: Vec<WeightedSegment<'a>>) {
        for segment in segments {
            if segment.segment.from == "http://www.openstreetmap.org/node/3182200179" {
                eprintln!("{:?}", segment);
            }
            if segment.segment.to == "http://www.openstreetmap.org/node/3182200179" {
                eprintln!("{:?}", segment);
            }

            if segment.segment.from == "http://www.openstreetmap.org/node/1428406498" {
                eprintln!("{:?}", segment);
            }
            if segment.segment.to == "http://www.openstreetmap.org/node/1428406498" {
                eprintln!("{:?}", segment);
            }

            let from_label = self.get_label_mut(segment.segment.from);
            let to_label = self.get_label_mut(segment.segment.to);
            let edge = Edge {
                node: to_label,
                cost: segment.weight as i64,
            };

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

    pub fn necessary_nodes(&self, from: &str, to: Vec<&String>, known_nodes: &mut BTreeSet<String>) {
        if self.get_label(from).is_none() {
            return;
        }
        let from_label = *self.get_label(from).unwrap();
        let tree = self.query_one_to_many(from, &to);

        for to_id in to {
            if self.get_label(to_id).is_none() {
                continue;
            }
            let mut current_label = *self.get_label(to_id).unwrap();
            known_nodes.insert(self.ids[current_label].to_owned());
            while tree[current_label] != from_label {
                current_label = tree[current_label];
                known_nodes.insert(self.ids[current_label].to_owned());
            }
            known_nodes.insert(self.ids[from_label].to_owned());
        }
    }

    fn query_one_to_many(&self, from: &str, to: &[&String]) -> Vec<usize> {
        let from_label = *self.get_label(from).unwrap();
        let mut to_labels: HashSet<usize> = to
            .iter()
            .map(|id| self.get_label(id))
            .filter(|id| id.is_some())
            .map(|id| *id.unwrap())
            .collect();
        let mut dist = vec![std::i64::MIN; self.adj_list.len()];
        let mut previous = vec![from_label; self.adj_list.len()];
        let mut queue = priority_queue::PriorityQueue::new();

        dist[from_label] = 0;
        queue.push(from_label, 0);

        while let Some((position, cost)) = queue.pop() {
            to_labels.remove(&position);

            if to_labels.is_empty() {
                break;
            }

            for edge in &self.adj_list[position] {
                let next_position = edge.node;
                let next_cost = cost - edge.cost;

                if next_cost > dist[next_position] {
                    queue.push(next_position, next_cost);
                    dist[next_position] = next_cost;
                    previous[next_position] = position;
                }
            }
        }

        previous
    }
}
