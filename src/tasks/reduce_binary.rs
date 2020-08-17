use crate::entities::weighted_tile::Location;
use crate::entities::tile::Tile;
use crate::io::tiles::load_tile;
use crate::entities::weighted_tile::{DirectedEdge, WeightedTile};
use crate::entities::profile::Profile;
use crate::entities::{segment::Segment, tile_coord::TileCoordinate};
use std::collections::BTreeMap;

pub fn create_binary_tile<'a>(
    root_dir: &str,
    coord: &'a TileCoordinate,
    profile: &Profile,
) -> WeightedTile {
    let base_tile = load_tile(coord, root_dir);
    let mut locations = Vec::new();
    let mut labels = BTreeMap::new();
    let mut edges = Vec::new();

    if let Ok(tile) = base_tile {  
        let Tile {nodes, ways, .. } = tile;
        let mut reverse_labels = BTreeMap::new();

        for id in nodes.keys() {
            let label = labels.len();
            reverse_labels.insert(label, id.clone());
            labels.insert(id.to_owned(), label);
        }

        for way in ways.values() {
            if !profile.has_access(way) {
                continue;
            }

            for edge in way.get_segments() {
                let Segment { from, to } = edge;
                let from_node = nodes.get(from).expect("Corrupted tile");
                let to_node = nodes.get(to).expect("Corrupted tile");

                if profile.is_obstacle(from_node) || profile.is_obstacle(to_node) {
                    continue;
                }

                let to_label = labels.get(edge.to).unwrap();
                let from_label = labels.get(edge.from).unwrap();

                if !profile.is_one_way(way) {
                    let backward_cost = profile.get_cost(to_node, from_node, way);
                    edges.push(DirectedEdge::new(*to_label, *from_label, backward_cost as u64));
                }

                let forward_cost = profile.get_cost(from_node, to_node, way);
                edges.push(DirectedEdge::new(*from_label, *to_label, forward_cost as u64));
            }
        }

        for id in reverse_labels.values() {
            let node = nodes.get(id).unwrap();
            let location = Location {id: node.get_id().to_owned(), lat: node.get_lat(), lon: node.get_long()};
            locations.push(location);
        }
    }
    WeightedTile::new(locations, labels, edges)
}