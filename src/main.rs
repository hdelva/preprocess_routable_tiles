#![recursion_limit = "128"]

extern crate clap;
use crate::io::profile::load_bicycle_profile;
use crate::tasks::merge_tiles::create_merged_tile;
use crate::tasks::reduce_profile::create_profile_tile;
use crate::tasks::reduce_transit::create_transit_tile;
use clap::{App, AppSettings, Arg, SubCommand};

mod entities;
mod io;
mod tasks;
mod util;

extern crate radix_heap;
extern crate serde;
extern crate serde_json;

use crate::entities::tile::Tile;
use crate::entities::tile_coord::TileCoordinate;
use crate::io::profile::load_car_profile;
use crate::io::profile::load_pedestrian_profile;
use crate::io::tiles::write_derived_tile;
use crate::io::get_tile_path;
use crate::util::get_tile_coords;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

// fetches all tiles in a bounding box
fn load_tiles(
    root: &str,
    lats: [f64; 2],
    lons: [f64; 2],
    zoom: u32,
) -> BTreeMap<TileCoordinate, Tile> {
    let todo = get_tile_coords(lats, lons, zoom);
    let parse_bar = ProgressBar::new(todo.len() as u64);
    parse_bar.set_style(
        ProgressStyle::default_bar()
            .template(
                "Loading Tiles [{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .progress_chars("█▓░"),
    );

    // fetch all tiles concurrently
    // converts a Result into an Option because we know some tiles are missing
    let parsed_tiles: Vec<Option<Tile>> = todo
        .into_par_iter()
        .map(|coordinate| {
            let path = get_tile_path(root, &coordinate);
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

fn main() {
    let matches = App::new("Routable Tiles Preprocessing")
        .version("1.0")
        .author("Harm Delva <harm.delva@ugent.be>")
        .about("Creates reduced versions of routable tiles (OpenStreetMap).")
        .setting(AppSettings::SubcommandRequiredElseHelp)
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
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input_dir")
                .help("Root directory to process of input files")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output_dir")
                .help("Root directory to write results to")
                .required(true)
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("reduce_profile")
                .about("Only retain tags that are relevant for the given profile")
                .arg(
                    Arg::with_name("profile")
                        .short("p")
                        .long("profile")
                        .value_name("car|bicycle|pedestrian")
                        .help("Sets the profile to use")
                        .possible_values(&["car", "bicycle", "pedestrian"])
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("reduce_transit")
                .about("Only retain elements that are necessary to traverse a tile")
                .arg(
                    Arg::with_name("profile")
                        .short("p")
                        .long("profile")
                        .value_name("car|bicycle|pedestrian")
                        .help("Sets the profile to use")
                        .possible_values(&["car", "bicycle", "pedestrian"])
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("merge")
                .about("Merge routable tiles into tiles of a higher zoom level"),
        )
        .get_matches();

    let zoom = matches
        .value_of("zoom")
        .unwrap()
        .parse::<u32>()
        .expect("Invalid zoom level");

    let [lats, lons] = match matches.value_of("area").unwrap() {
        "belgium" => [[49., 52.], [2.25, 6.6]],
        "dummy" => [[51.15, 51.25], [4.40, 4.5]],
        _ => unreachable!(),
    };

    let input_dir = matches.value_of("input").unwrap();
    let output_dir = matches.value_of("output").unwrap();

    match matches.subcommand_name() {
        Some("reduce_profile") => {
            let profile_name = matches
                .subcommand_matches("reduce_profile")
                .expect("Subcommand arguments are missing")
                .value_of("profile")
                .unwrap();
            let profile = match profile_name {
                "car" => load_car_profile().unwrap(),
                "pedestrian" => load_pedestrian_profile().unwrap(),
                "bicycle" => load_bicycle_profile().unwrap(),
                _ => unreachable!(),
            };

            println!("Used concepts: {:?}", profile.get_used_concepts());

            let index = load_tiles(input_dir, lats, lons, zoom);
            let bar = ProgressBar::new(index.len() as u64);
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("Pruning Tags [{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} {msg}")
                    .progress_chars("█▓░"),
            );

            index.par_iter().for_each(|(id, _)| {
                let profile_tile_path = get_tile_path(output_dir, id);
                let profile_tile = create_profile_tile(&index, id, &profile);
                write_derived_tile(profile_tile, &profile_tile_path);
                bar.inc(1);
            });

            bar.finish();
        }
        Some("reduce_transit") => {
            let profile_name = matches
                .subcommand_matches("reduce_transit")
                .expect("Subcommand arguments are missing")
                .value_of("profile")
                .unwrap();
            let profile = match profile_name {
                "car" => load_car_profile().unwrap(),
                "pedestrian" => load_pedestrian_profile().unwrap(),
                "bicycle" => load_bicycle_profile().unwrap(),
                _ => unreachable!(),
            };

            let index = load_tiles(input_dir, lats, lons, zoom);
            let bar = ProgressBar::new(index.len() as u64);
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("Pruning Ways [{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} {msg}")
                    .progress_chars("█▓░"),
            );

            index.par_iter().for_each(|(id, _)| {
                let profile_tile_path = get_tile_path(output_dir, id);
                let profile_tile = create_transit_tile(&index, id, &profile);
                write_derived_tile(profile_tile, &profile_tile_path);
                bar.inc(1);
            });

            bar.finish();
        }
        Some("merge") => {
            let index = load_tiles(input_dir, lats, lons, zoom);

            let mut todo = BTreeSet::new();
            for (source_coord, _) in index.iter() {
                todo.insert(source_coord.get_parent());
            }

            let bar = ProgressBar::new(todo.len() as u64);
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("Merging tiles [{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} {msg}")
                    .progress_chars("█▓░"),
            );

            todo.par_iter().for_each(|id| {
                let merged_tile_path = get_tile_path(output_dir, id);
                let c = id.get_children();
                let merged_tile = create_merged_tile(&index, &c, id);
                write_derived_tile(merged_tile, &merged_tile_path);
                bar.inc(1);
            });

            bar.finish();
        }
        _ => unreachable!(),
    };
}
