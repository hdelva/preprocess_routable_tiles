use crate::io::tiles::load_cached_tile;
use crate::entities::graph::Graph;
use crate::entities::node::Node;
use crate::entities::profile::Profile;
use crate::entities::tile::DerivedTile;
use crate::entities::tile::Tile;
use crate::entities::tile_coord::TileCoordinate;
use crate::entities::way::Way;
use crate::io::tiles::load_tile;
use crate::util::deg2num;
use crate::util::edge_nodes::get_edge_nodes;
use crate::util::get_tile_edges;
use crate::util::num2deg;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

fn adjacent_tiles(coord: &TileCoordinate, target_zoom: u32) -> Vec<TileCoordinate> {
    let [top, left] = num2deg(coord.x, coord.y, coord.zoom);
    let base_coord = deg2num(top, left, target_zoom);

    let n = 2u32.pow(target_zoom - coord.zoom);

    let mut result = Vec::new();
    for x in base_coord.x - 1..=base_coord.x + n {
        // top row, including corners
        result.push(TileCoordinate::new(x, base_coord.y - 1, target_zoom));

        // bottom row, including corners
        result.push(TileCoordinate::new(x, base_coord.y + n, target_zoom));
    }

    for y in base_coord.y..base_coord.y + n {
        // left column
        result.push(TileCoordinate::new(base_coord.x - 1, y, target_zoom));

        // left column
        result.push(TileCoordinate::new(base_coord.x + n, y, target_zoom));
    }

    result
}

fn update_graph<'a>(graph: &mut Graph<'a>, tile: &'a Tile, profile: &Profile) {
    let weights = tile.get_weighted_segments(profile);
    graph.add_edges(weights);
}

pub fn create_indirect_transit_tile<'a>(
    root_dir: &str,
    padding_zoom: u32,
    coord: &'a TileCoordinate,
    profile: &Profile,
) -> DerivedTile {
    // build base graph
    let base_tile = load_cached_tile(coord, root_dir);
    let mut reduced_ways = BTreeMap::new();
    let mut reduced_nodes = BTreeMap::new();

    if let Ok(tile) = base_tile {
        let weights = tile.get_weighted_segments(profile);
        let mut graph = Graph::new(weights);

        // collect neighboring tiles
        let neighbor_coords = adjacent_tiles(coord, padding_zoom);
        let neighbors: Vec<Tile> = neighbor_coords
            .iter()
            .map(|v| load_cached_tile(v, root_dir))
            .filter(|v| v.is_ok())
            .map(|v| v.unwrap())
            .collect();

        // add neighboring tile data
        neighbors
            .iter()
            .for_each(|t| update_graph(&mut graph, t, profile));

        // squeeze out the bounds of the edge tiles
        // bit of a shotgun approach, but should work
        let all_bounds: Vec<[f64; 4]> = neighbor_coords.iter().map(|c| get_tile_edges(c)).collect();
        let [e, _n, _w, _s] = all_bounds
            .iter()
            .max_by(|x, y| x[0].partial_cmp(&y[0]).unwrap()) // floats don't have a total order
            .unwrap();
        let [_e, n, _w, _s] = all_bounds
            .iter()
            .max_by(|x, y| x[1].partial_cmp(&y[1]).unwrap())
            .unwrap();
        let [_e, _n, w, _s] = all_bounds
            .iter()
            .min_by(|x, y| x[2].partial_cmp(&y[2]).unwrap())
            .unwrap();
        let [_e, _n, _w, s] = all_bounds
            .iter()
            .min_by(|x, y| x[3].partial_cmp(&y[3]).unwrap())
            .unwrap();

        let mut edge_nodes = BTreeSet::new();
        edge_nodes.extend(get_edge_nodes(&tile, [*e, *n, *w, *s]));
        for neighbor in neighbors.iter() {
            edge_nodes.extend(get_edge_nodes(neighbor, [*e, *n, *w, *s]));
        }

        let mut all_nodes = BTreeMap::new();
        for node in tile.get_nodes().values() {
            all_nodes.insert(node.get_id().to_owned(), node.clone());
        }
        for neighbor in neighbors.iter() {
            for node in neighbor.get_nodes().values() {
                all_nodes.insert(node.get_id().to_owned(), node.clone());
            }
        }

        let mut necessary_nodes = BTreeSet::new();
        for first_node in edge_nodes.iter() {
            graph.necessary_nodes(
                first_node,
                edge_nodes
                    .iter()
                    .filter(|id| *id != first_node)
                    .map(|id| id)
                    .collect(),
                    &mut necessary_nodes,
            );
        }

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
                let nodes: Vec<String> = way.get_nodes()[first_node..=last_node].into();
                for node_id in nodes.iter() {
                    let node: Node = tile.get_nodes()[node_id].clone();
                    reduced_nodes.insert(node_id.clone(), node);
                }

                let way = Way::new(
                    way_id.clone(),
                    nodes,
                    None,
                    *way.get_max_speed(),
                    way.get_tags().clone(),
                    way.get_undefined_tags().to_vec(),
                );
                reduced_ways.insert(way_id.clone(), way);
            }
        }
    }

    DerivedTile::new(*coord, reduced_nodes, reduced_ways)
}

pub fn create_transit_tile<'a>(
    root_dir: &str,
    coord: &'a TileCoordinate,
    profile: &Profile,
) -> DerivedTile {
    let base_tile = load_tile(coord, root_dir);
    let mut reduced_ways = BTreeMap::new();
    let mut reduced_nodes = BTreeMap::new();

    if let Ok(tile) = base_tile {  
        let weights = tile.get_weighted_segments(profile);
        let graph = Graph::new(weights);

        let bounds = get_tile_edges(coord);
        let edge_nodes = get_edge_nodes(&tile, bounds);
        let mut necessary_nodes = BTreeSet::new();
        for first_node in edge_nodes.iter() {
            graph.necessary_nodes(
                first_node,
                edge_nodes
                    .iter()
                    .filter(|id| *id != first_node)
                    .map(|id| id)
                    .collect(),
                    &mut necessary_nodes,
            );
        }

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
                let nodes: Vec<String> = way.get_nodes()[first_node..=last_node].into();
                for node_id in nodes.iter() {
                    let node: Node = tile.get_nodes()[node_id].clone();
                    reduced_nodes.insert(node_id.clone(), node);
                }

                let way = Way::new(
                    way_id.clone(),
                    nodes,
                    None,
                    *way.get_max_speed(),
                    way.get_tags().clone(),
                    way.get_undefined_tags().to_vec(),
                );
                reduced_ways.insert(way_id.clone(), way);
            }
        }
    }

    DerivedTile::new(*coord, reduced_nodes, reduced_ways)
}
