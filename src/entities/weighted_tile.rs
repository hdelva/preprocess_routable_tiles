use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct WeightedTile {
    pub locations: Vec<Location>,
    pub labels: BTreeMap<String, usize>,
    pub edges: Vec<DirectedEdge>,
}

impl WeightedTile {
    pub fn new(
        locations: Vec<Location>, 
        labels: BTreeMap<String, usize>,
        edges: Vec<DirectedEdge>
    ) -> WeightedTile {
        WeightedTile {
            locations,
            labels,
            edges,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DirectedEdge {
    pub from: usize,
    pub to: usize,
    pub weight: u64,
}


impl DirectedEdge {
    pub fn new(from: usize, to: usize, weight: u64) -> DirectedEdge {
        DirectedEdge { from, to, weight }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Location {
    pub lat: f64,
    pub lon: f64, 
    pub id: String,
}
