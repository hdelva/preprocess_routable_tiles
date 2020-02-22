use crate::entities::node::Node;
use crate::entities::profile::Profile;
use crate::entities::segment::{Segment, WeightedSegment};
use crate::entities::tile_coord::TileCoordinate;
use crate::entities::way::Way;
use std::collections::BTreeMap;

pub struct Tile {
    nodes: BTreeMap<String, Node>,
    ways: BTreeMap<String, Way>,
    coordinate: TileCoordinate,
}

impl Tile {
    pub fn new(
        coordinate: TileCoordinate,
        nodes: BTreeMap<String, Node>,
        ways: BTreeMap<String, Way>,
    ) -> Tile {
        Tile {
            coordinate,
            nodes,
            ways,
        }
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
        let mut result = vec![];
        for (_, way) in self.get_ways() {
            if !profile.has_access(way) {
                continue;
            }

            for edge in way.get_segments() {
                let Segment { from, to } = edge;
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

pub struct DerivedTile {
    nodes: BTreeMap<String, Node>,
    ways: BTreeMap<String, Way>,
    coordinate: TileCoordinate,
}

impl DerivedTile {
    pub fn new(
        coordinate: TileCoordinate,
        nodes: BTreeMap<String, Node>,
        ways: BTreeMap<String, Way>,
    ) -> DerivedTile {
        DerivedTile {
            nodes,
            ways,
            coordinate,
        }
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
}
