use crate::entities::graph::Graph;
use crate::entities::node::Node;
use crate::entities::profile::Profile;
use crate::entities::tile::DerivedTile;
use crate::entities::tile::Tile;
use crate::entities::tile_coord::TileCoordinate;
use crate::entities::way::Way;
use crate::util::edge_nodes::get_edge_nodes;
use crate::util::get_tile_edges;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

fn create_graph<'a>(
    index: &'a BTreeMap<TileCoordinate, Tile>,
    coord: &TileCoordinate,
    profile: &Profile,
) -> Graph<'a> {
    let mut weights = vec![];
    if let Some(other) = index.get(coord) {
        weights.extend(other.get_weighted_segments(profile));
    }

    return Graph::new(weights);
}

pub fn create_transit_tile<'a>(
    index: &'a BTreeMap<TileCoordinate, Tile>,
    coord: &'a TileCoordinate,
    profile: &Profile,
) -> DerivedTile {
    let tile = index.get(coord).expect("Inconsistent data");
    let bounds = get_tile_edges(coord);
    let graph = create_graph(&index, coord, profile);

    let edge_nodes = get_edge_nodes(tile, bounds);
    let mut necessary_nodes = BTreeSet::new();
    for first_node in edge_nodes.iter() {
        let mut result = graph.necessary_nodes(
            first_node,
            edge_nodes
                .iter()
                .filter(|id| *id != first_node)
                .map(|id| *id)
                .collect(),
        );
        necessary_nodes.append(&mut result);
    }

    let mut reduced_ways = BTreeMap::new();
    let mut reduced_nodes = BTreeMap::new();
    for (way_id, way) in tile.get_ways() {
        let mut first_node = way.get_nodes().len();
        let mut last_node = 0;
        for (location, node_id) in way.get_nodes().iter().enumerate() {
            if necessary_nodes.contains(node_id) {
                first_node = std::cmp::min(first_node, location);
                last_node = std::cmp::max(last_node, location);
            }
        }

        if last_node > first_node {
            let nodes: Vec<String> = way.get_nodes()[first_node..last_node + 1].into();
            for node_id in nodes.iter() {
                let node: Node = tile.get_nodes()[node_id].clone();
                reduced_nodes.insert(node_id.clone(), node);
            }

            let way = Way::new(
                way_id.clone(),
                nodes,
                way.get_max_speed().clone(),
                way.get_tags().clone(),
                way.get_undefined_tags().to_vec(),
            );
            reduced_ways.insert(way_id.clone(), way);
        }
    }

    DerivedTile::new(
        tile.get_coordinate().clone(),
        reduced_nodes,
        reduced_ways,
    )
}
