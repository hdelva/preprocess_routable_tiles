use crate::entities::node::Node;
use crate::entities::way::Way;
use std::collections::BTreeMap;
use crate::entities::tile_coord::TileCoordinate;
use crate::entities::segment::{Segment, WeightedSegment};
use crate::entities::profile::Profile;

pub struct Tile {
    id: String,
    nodes: BTreeMap<String, Node>,
    ways: BTreeMap<String, Way>,
    coordinate: TileCoordinate,
}

impl Tile {
    pub fn new(id: String, coordinate: TileCoordinate, nodes: BTreeMap<String, Node>, ways: BTreeMap<String, Way>) -> Tile {
        Tile {id, coordinate, nodes, ways}
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_nodes(&self) -> &BTreeMap<String, Node> {
        &self.nodes
    }

    pub fn get_ways(&self) -> &BTreeMap<String, Way> {
        &self.ways
    }

    pub fn get_coordinate(&self) -> &TileCoordinate {
        &self.coordinate
    }

    pub fn get_weighted_segments(&self, profile: &Profile) -> Vec<WeightedSegment> {
        let mut result = vec!();
        for (_, way) in self.get_ways() {
            if !profile.has_access(way) {
                continue;
            }

            for edge in way.get_segments() {
                let Segment {from, to} = edge;
                let from_node = self.get_nodes().get(from).expect("Corrupted tile");
                let to_node = self.get_nodes().get(to).expect("Corrupted tile");

                if profile.is_obstacle(from_node) || profile.is_obstacle(to_node) {
                    continue;
                }

                if !profile.is_one_way(way) {
                    let backward_cost = profile.get_cost(to_node, from_node, way);
                    let reverse_edge = Segment::new(edge.to.clone(), edge.from.clone());
                    result.push(WeightedSegment::new(reverse_edge, backward_cost as u64));
                }

                let forward_cost = profile.get_cost(from_node, to_node, way);
                result.push(WeightedSegment::new(edge, forward_cost as u64));
            }
        }
        return result;
    }
}

pub struct TransitTile<'a> {
    nodes: BTreeMap<String, Node>,
    ways: BTreeMap<String, Way>,
    original: &'a Tile
}

impl<'a> TransitTile<'a> {
    pub fn new(nodes:BTreeMap<String, Node>, ways:BTreeMap<String, Way>, original: &'a Tile) -> TransitTile {
        TransitTile {nodes, ways, original}
    }

    pub fn get_nodes(&self) -> &BTreeMap<String, Node>{
        &self.nodes
    }

    pub fn get_ways(&self) -> &BTreeMap<String, Way> {
        &self.ways
    }

    pub fn get_id(&self) -> &str {
        self.original.get_id()
    }

    pub fn get_coordinate(&self) -> &TileCoordinate {
        &self.original.get_coordinate()
    }
}