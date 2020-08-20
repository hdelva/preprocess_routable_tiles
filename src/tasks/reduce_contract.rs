use crate::io::tiles::load_tile;
use crate::entities::node::Node;
use crate::entities::tile::Tile;
use crate::entities::tile_coord::TileCoordinate;
use crate::entities::way::Way;
use crate::util::edge_nodes::get_edge_nodes;
use crate::util::get_tile_edges;
use crate::util::haversine::get_distance;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

fn get_node_degrees(tile: &Tile) -> BTreeMap<String, u32> {
    let mut result = BTreeMap::new();
    for way in tile.get_ways().values() {
        for node in way.get_nodes() {
            *result.entry(node.to_owned()).or_insert(0) += 1;
        }
    }
    result
}

fn get_useful_nodes(tile: &Tile, bounds: [f64; 4]) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    let degrees = get_node_degrees(tile);
    let oob = get_edge_nodes(tile, bounds);

    for way in tile.get_ways().values() {
        let way_nodes = way.get_nodes();

        for (i, node_id) in way_nodes.iter().enumerate() {
            let node = tile.get_nodes().get(node_id).unwrap();

            if oob.contains(node_id)
                || degrees.get(node_id).unwrap_or(&0) > &1
                || i > 0 && oob.contains(&way_nodes[i - 1])
                || i < way_nodes.len() - 1 && oob.contains(&way_nodes[i + 1])
                || i == 0
                || i == way_nodes.len() - 1
                || node.get_tags().get(&"osm:highway".to_owned()).is_some()
                || node.get_tags().get(&"osm:barrier".to_owned()).is_some()
                || node.get_tags().get(&"osm:crossing".to_owned()).is_some()
            {
                result.insert(node_id.to_string());
            }
        }
    }
    result
}

fn contract_way(way: &Way, tile: &Tile, useful_nodes: &BTreeSet<String>) -> Way {
    let mut nodes = Vec::new();
    let mut distances = Vec::new();

    let way_nodes = way.get_nodes();
    let mut previous_node = tile.get_nodes().get(&way_nodes[0]).unwrap();
    nodes.push(previous_node.get_id().to_string());
    let mut distance_since = 0;

    for node_id in way_nodes.iter().skip(1) {
        let current_node = tile.get_nodes().get(node_id).unwrap();
        distance_since += (get_distance(current_node, previous_node) * 1000.0).round() as i64;
        if useful_nodes.contains(current_node.get_id()) {
            nodes.push(current_node.get_id().to_string());
            distances.push(distance_since);
            distance_since = 0;
        }
        previous_node = current_node;
    }

    Way::new(
        way.get_id().to_string(),
        nodes,
        Some(distances),
        *way.get_max_speed(),
        way.get_tags().clone(),
        way.get_undefined_tags().to_vec(),
    )
}

// not used at the moment
// not sure if a tile with only shortcuts is useful
#[allow(dead_code)]
pub fn create_contracted_tile<'a>(
    root_dir: &str,
    coord: &'a TileCoordinate,
) -> Tile {
    let base_tile = load_tile(coord, root_dir);
    let bounds = get_tile_edges(coord);

    let mut reduced_ways: BTreeMap<String, Way> = BTreeMap::new();
    let mut reduced_nodes: BTreeMap<String, Node> = BTreeMap::new();
    if let Ok(tile) = base_tile {
        let useful_nodes = get_useful_nodes(&tile, bounds);

        for (node_id, node) in tile.get_nodes() {
            if useful_nodes.contains(node_id) {
                reduced_nodes.insert(node_id.to_string(), node.clone());
            }
        }

        for (way_id, way) in tile.get_ways() {
            let new_way = contract_way(way, &tile, &useful_nodes);
            reduced_ways.insert(way_id.clone(), new_way);
        }
    }

    Tile::new(*coord, reduced_nodes, reduced_ways)
}
