#![recursion_limit="128"]

mod io;
mod entities;
mod util;

extern crate serde;
extern crate serde_json;
extern crate radix_heap;

use crate::util::{get_tile_coords, get_tile_edges};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use crate::entities::tile_coord::TileCoordinate;
use crate::entities::tile::{Tile, TransitTile};
use crate::entities::graph::Graph;
use crate::entities::way::Way;
use crate::entities::node::Node;
use crate::io::tiles::write_transit_tile;
use crate::io::{get_tile_path, get_transit_tile_path};
use crate::io::profile::load_profile;
use crate::entities::profile::Profile;

fn load_tiles(lats: [f64; 2], lons: [f64; 2]) -> BTreeMap<TileCoordinate, Tile> {
    let todo = get_tile_coords(lats, lons, 8);
    let parse_bar = ProgressBar::new(todo.len() as u64);
    parse_bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("█▓░"));

    let parsed_tiles: Vec<Option<Tile>> = todo.into_par_iter().map(|coordinate| {
        let path = get_tile_path(&coordinate);
        parse_bar.inc(1);
        let result =  io::tiles::load_tile(coordinate, &path);

        if let Ok(tile) = result {
            return Some(tile)
        }

        return None
    }).collect();

    let mut index = BTreeMap::new();
    for optional_tile in parsed_tiles.into_iter() {
        if let Some(tile) = optional_tile {
            index.insert(tile.get_coordinate().clone(), tile);
        }
    }
    parse_bar.finish();
    return index;
}

fn create_graph<'a>(
    index: &'a BTreeMap<TileCoordinate, Tile>,
    coord: &TileCoordinate,
    profile: &Profile
) -> Graph<'a> {
    let mut weights  = vec!();
    for i in 0..3 {
        for j in 0..3 {
            let other_coord = TileCoordinate::new(
                coord.x + i - 1,
                coord.y + j - 1,
                coord.zoom
            );
            if let Some(other) = index.get(&other_coord) {
                weights.extend(other.get_weighted_segments(profile));
            }
        }
    }

    return Graph::new( weights);
}

fn get_edge_nodes(tile: &Tile, bounds: [f64; 4]) -> BTreeSet<&str> {
    let [e, n, w, s] = bounds;
    let mut oob = BTreeSet::new();
    for node in tile.get_nodes().values() {
        let lat = node.get_lat();
        let long = node.get_long();

        if !(s <= lat && lat <= n) || !(w <= long && long <= e) {
            oob.insert(node.get_id());
        }
    }
    return oob
}

fn create_transit_tile<'a>(
    index: &'a BTreeMap<TileCoordinate, Tile>,
    coord: &'a TileCoordinate,
    profile: &Profile
) -> TransitTile<'a> {
    let tile = index.get(coord).expect("Inconsistent data");
    let bounds = get_tile_edges(coord);
    let graph = create_graph(&index, coord, profile);

    let edge_nodes = get_edge_nodes(tile, bounds);
    let mut necessary_nodes = BTreeSet::new();
    for first_node in edge_nodes.iter() {
        let mut result = graph.necessary_nodes(
            first_node,
            edge_nodes.iter().filter(|id| *id != first_node)
                .map(|id| *id).collect());
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
            let nodes: Vec<String> = way.get_nodes()[first_node .. last_node + 1].into();
            for node_id in nodes.iter() {
                let node: Node = tile.get_nodes()[node_id].clone();
                reduced_nodes.insert(node_id.clone(), node);
            }

            let way = Way::new(
                way_id.clone(),
                nodes,
                way.get_max_speed().clone(),
                way.get_tags().clone(),
                way.get_undefined_tags().to_vec()
            );
            reduced_ways.insert(way_id.clone(), way);
        }
    }

    TransitTile::new(reduced_nodes, reduced_ways, tile)
}

fn main() {
    let lats = [49., 52.];
    let lons = [2.25, 6.6];

    /*
    let lats = [51.15, 51.25];
    let lons = [4.40, 4.5];
    */

    let profile = load_profile().unwrap();

    let index = load_tiles(lats, lons);

    let bar = ProgressBar::new(index.len() as u64);
    bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("█▓░"));

    index.par_iter().for_each(|(id, _)| {
        //if id.x == 8393 && id.y == 5469 {
            let transit_tile_path = get_transit_tile_path(id);
            let transit_tile = create_transit_tile(&index, id, &profile);
            write_transit_tile(transit_tile, &transit_tile_path);
            bar.inc(1);
        //}
    });

    bar.finish();
}
