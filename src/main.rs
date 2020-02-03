#![recursion_limit = "128"]

extern crate clap;
use clap::{App, Arg, SubCommand, AppSettings};

mod entities;
mod io;
mod util;

extern crate radix_heap;
extern crate serde;
extern crate serde_json;

use crate::entities::graph::Graph;
use crate::entities::node::Node;
use crate::entities::profile::Profile;
use crate::entities::segment::Segment;
use crate::entities::tile::{DerivedTile, Tile};
use crate::entities::tile_coord::TileCoordinate;
use crate::entities::way::Way;
use crate::io::get_profile_tile_path;
use crate::io::profile::load_car_profile;
use crate::io::profile::load_pedestrian_profile;
use crate::io::tiles::write_derived_tile;
use crate::io::{get_tile_path, get_transit_tile_path};
use crate::util::{get_tile_coords, get_tile_edges};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

// fetches all tiles in a bounding box
fn load_tiles(lats: [f64; 2], lons: [f64; 2], zoom: u32) -> BTreeMap<TileCoordinate, Tile> {
    let todo = get_tile_coords(lats, lons, zoom);
    let parse_bar = ProgressBar::new(todo.len() as u64);
    parse_bar.set_style(
        ProgressStyle::default_bar()
            .template("Loading Tiles [{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} {msg}")
            .progress_chars("█▓░"),
    );

    // fetch all tiles concurrently
    // converts a Result into an Option because we know some tiles are missing
    let parsed_tiles: Vec<Option<Tile>> = todo
        .into_par_iter()
        .map(|coordinate| {
            let path = get_tile_path(&coordinate);
            parse_bar.inc(1);
            return io::tiles::load_tile(coordinate, &path).ok();
        })
        .collect();

    // todo: filter out unnecessary roads/nodes
    let mut index = BTreeMap::new();
    for optional_tile in parsed_tiles.into_iter() {
        if let Some(tile) = optional_tile {
            index.insert(tile.get_coordinate().clone(), tile);
        }
    }

    parse_bar.finish();
    return index;
}

// create weighted edge graph around a given tile
fn create_graph<'a>(
    index: &'a BTreeMap<TileCoordinate, Tile>,
    coord: &TileCoordinate,
    profile: &Profile,
) -> Graph<'a> {
    let mut weights = vec![];
    for i in 0..3 {
        for j in 0..3 {
            let other_coord = TileCoordinate::new(coord.x + i - 1, coord.y + j - 1, coord.zoom);
            if let Some(other) = index.get(&other_coord) {
                weights.extend(other.get_weighted_segments(profile));
            }
        }
    }

    return Graph::new(weights);
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
    return oob;
}

fn create_profile_tile<'a>(
    index: &'a BTreeMap<TileCoordinate, Tile>,
    coord: &'a TileCoordinate,
    profile: &Profile,
) -> DerivedTile<'a> {
    let tile = index.get(coord).expect("Inconsistent data");
    let mut reduced_ways: BTreeMap<String, Way> = BTreeMap::new();
    let mut reduced_nodes: BTreeMap<String, Node> = BTreeMap::new();
    let concepts = profile.get_used_concepts();

    for (way_id, way) in tile.get_ways() {
        if profile.has_access(way) {
            let mut new_tags = BTreeMap::new();
            for tag in way.get_tags().iter() {
                if concepts.contains(tag.0) && concepts.contains(tag.1) {
                    new_tags.insert(tag.0.to_owned(), tag.1.to_owned());
                }
            }
            if let Some(name) = way.get_tags().get("osm:name") {
                new_tags.insert("osm:name".to_owned(), name.to_string());
            }
            let new_way = Way::new(
                way_id.to_owned(),
                way.get_nodes().to_owned(),
                way.get_max_speed().clone(),
                new_tags,
                Vec::new(),
            );

            reduced_ways.insert(way_id.clone(), new_way);

            for edge in way.get_segments() {
                let Segment { from, to } = edge;
                let from_node = tile.get_nodes().get(from).expect("Corrupted tile");
                let to_node = tile.get_nodes().get(to).expect("Corrupted tile");
                reduced_nodes.insert(from.to_string(), from_node.clone());
                reduced_nodes.insert(to.to_string(), to_node.clone());
            }
        }
    }
    return DerivedTile::new(reduced_nodes, reduced_ways, tile);
}

fn create_transit_tile<'a>(
    index: &'a BTreeMap<TileCoordinate, Tile>,
    coord: &'a TileCoordinate,
    profile: &Profile,
) -> DerivedTile<'a> {
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

    DerivedTile::new(reduced_nodes, reduced_ways, tile)
}

fn main() {
    let matches = App::new("Routable Tiles Preprocessing")
        .version("1.0")
        .author("Harm Delva <harm.delva@ugent.be>")
        .about("Creates reduced versions of routable tiles (OpenStreetMap).")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("profile")
                .short("p")
                .long("profile")
                .value_name("car|pedestrian")
                .help("Sets the profile to use")
                .possible_values(&["car", "pedestrian"])
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("area")
                .short("a")
                .long("area")
                .value_name("belgium|dummy")
                .help("Sets the bounding box")
                .possible_values(&["belgium", "dummy"])
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("zoom")
                .short("z")
                .long("zoom")
                .help("Sets the zoom level")
                .required(true)
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("reduce_profile")
                .about("Only retain tags that are relevant for the given profile")
        )
        .subcommand(
            SubCommand::with_name("reduce_transit")
                .about("Only retain elements that are necessary to traverse a tile")
        )
        .get_matches();

    let zoom = matches.value_of("zoom").unwrap().parse::<u32>().expect("Invalid zoom level");

    let [lats, lons] = match matches.value_of("area").unwrap() {
        "belgium" => [[49., 52.], [2.25, 6.6]],
        "dummy" => [[51.15, 51.25], [4.40, 4.5]],
        _ => unreachable!(),
    };

    let profile_name = matches.value_of("profile").unwrap();
    let profile = match profile_name {
        "car" => load_car_profile().unwrap(),
        "pedestrian" => load_pedestrian_profile().unwrap(),
        _ => unreachable!(),
    };

    let index = load_tiles(lats, lons, zoom);
    let bar = ProgressBar::new(index.len() as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} {msg}")
            .progress_chars("█▓░"),
    );

    match matches.subcommand_name() {
        Some("reduce_profile") => {
            println!("Used concepts: {:?}", profile.get_used_concepts());

            index.par_iter().for_each(|(id, _)| {
                let profile_tile_path = get_profile_tile_path(profile_name, id);
                let profile_tile = create_profile_tile(&index, id, &profile);
                write_derived_tile(profile_tile, &profile_tile_path);
                bar.inc(1);
            });
        },
        Some("reduce_transit") => {
            index.par_iter().for_each(|(id, _)| {
                let profile_tile_path = get_transit_tile_path(profile_name, id);
                let profile_tile = create_transit_tile(&index, id, &profile);
                write_derived_tile(profile_tile, &profile_tile_path);
                bar.inc(1);
            });
        },
        _ => unreachable!(),
    };

    bar.finish();
}
