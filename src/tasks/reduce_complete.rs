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

fn create_graph<'a>(
    index: &'a BTreeMap<TileCoordinate, Tile>,
    coord: &TileCoordinate,
    profile: &Profile,
) -> Graph<'a> {
    let mut weights = vec![];
    if let Some(other) = index.get(coord) {
        weights.extend(other.get_weighted_segments(profile));
    }

    Graph::new(weights)
}

pub fn create_complete_tile<'a>(
    index: &'a BTreeMap<TileCoordinate, Tile>,
    coord: &'a TileCoordinate,
    profile: &Profile,
) -> DerivedTile {
    let tile = index.get(coord).expect("Inconsistent data");
    let bounds = get_tile_edges(coord);
    let graph = create_graph(&index, coord, profile);

    let edge_nodes = get_edge_nodes(tile, bounds);
    let mut costs = BTreeMap::new();
    for first_node in edge_nodes.iter() {
        let result = graph.query_costs(
            first_node,
            edge_nodes
                .iter()
                .filter(|id| *id != first_node)
                .map(|id| id)
                .collect(),
        );
        for (other_node, cost) in result.into_iter() {
            costs.insert((first_node.clone(), other_node), cost);
        }
    }

    let mut reduced_ways = BTreeMap::new();
    let mut reduced_nodes = BTreeMap::new();

    for node_id in edge_nodes.into_iter() {
        let node: Node = tile.get_nodes()[&node_id].clone();
        reduced_nodes.insert(node_id.clone(), node);
    }

    for ((from_id, to_id), cost) in costs.into_iter() {
        let way_id = format!(
            "http://hdelva.be/cway/{}_{}_{}_{}", 
            coord.zoom, 
            coord.x, 
            coord.y, 
            reduced_ways.len()
        );
        let way = Way::new(
            way_id,
            vec!(from_id, to_id),
            Some(vec!(cost)),
            None,
            BTreeMap::new(),
            vec!(),
        );
        reduced_ways.insert(way.get_id().to_owned(), way);
    }

    DerivedTile::new(*tile.get_coordinate(), reduced_nodes, reduced_ways)
}
