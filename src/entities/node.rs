use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Node {
    id: String,
    lat: f64,
    long: f64,
    tags: BTreeMap<String, String>,
    undefined_tags: Vec<String>,
}

impl Node {
    pub fn new(
        id: String,
        lat: f64,
        long: f64,
        tags: BTreeMap<String, String>,
        undefined_tags: Vec<String>,
    ) -> Node {
        return Node {id, lat, long, tags, undefined_tags};
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_lat(&self) -> f64 {
        self.lat
    }

    pub fn get_long(&self) -> f64 {
        self.long
    }

    pub fn get_tags(&self) -> &BTreeMap<String, String> {
        &self.tags
    }

    pub fn get_undefined_tags(&self) -> &[String] {
        &self.undefined_tags
    }
}