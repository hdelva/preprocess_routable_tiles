use std::collections::BTreeMap;
use crate::entities::segment::Segment;

#[derive(Debug, Clone)]
pub struct Way {
    id: String,
    nodes: Vec<String>,
    distances: Option<Vec<i64>>,
    max_speed: Option<f64>,
    tags: BTreeMap<String, String>,
    undefined_tags: Vec<String>,
}

impl Way {
    pub fn new(
        id: String,
        nodes: Vec<String>,
        distances: Option<Vec<i64>>,
        max_speed: Option<f64>,
        tags: BTreeMap<String, String>,
        undefined_tags: Vec<String>,
    ) -> Way {
        Way {id, nodes, distances, max_speed, tags, undefined_tags}
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_nodes(&self) -> &[String] {
        &self.nodes
    }

    pub fn get_distances(&self) -> &Option<Vec<i64>> {
        &self.distances
    }

    pub fn get_max_speed(&self) -> &Option<f64> {
        &self.max_speed
    }

    pub fn get_tags(&self) -> &BTreeMap<String, String> {
        &self.tags
    }

    pub fn get_undefined_tags(&self) -> &[String] {
        &self.undefined_tags
    }

    pub fn get_segments(&self) -> Vec<Segment> {
        let mut result = vec!();
        for i in 0 .. self.get_nodes().len() - 1 {
            result.push(Segment::new(&self.get_nodes()[i],  &self.get_nodes()[i + 1]));
        }
        result
    }
}