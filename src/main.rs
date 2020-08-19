#![recursion_limit = "128"]

extern crate clap;
#[macro_use] extern crate cached;

use crate::tasks::load_tile::fetch_tile;
use crate::io::tiles::write_flexbuffers_tile;
use crate::tasks::reduce_binary::create_binary_tile;
use crate::io::get_binary_tile_path;
use crate::io::profile::load_bicycle_profile;
use crate::tasks::merge_tiles::create_merged_tile;
use crate::tasks::reduce_contract::create_contracted_tile;
use crate::tasks::reduce_profile::create_profile_tile;
use crate::tasks::reduce_transit::create_indirect_transit_tile;
use crate::tasks::reduce_transit::create_transit_tile;
use clap::{App, AppSettings, Arg};

mod entities;
mod io;
mod tasks;
mod util;

extern crate priority_queue;
extern crate serde;
extern crate serde_json;

use crate::io::get_tile_path;
use crate::io::profile::load_car_profile;
use crate::io::profile::load_pedestrian_profile;
use crate::io::tiles::write_derived_tile;
use crate::util::get_tile_coords;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::str::FromStr;
use entities::tile_coord::TileCoordinate;

enum Areas {
    Belgium,
    London,
    Pyrenees,
    Dummy,
}

impl FromStr for Areas {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "belgium" => Ok(Areas::Belgium),
            "london" => Ok(Areas::London),
            "pyrenees" => Ok(Areas::Pyrenees),
            "dummy" => Ok(Areas::Dummy),
            _ => Err("no match"),
        }
    }
}

fn main() {
    let matches = App::new("Routable Tiles Preprocessing")
        .version("1.0")
        .author("Harm Delva <harm.delva@ugent.be>")
        .about("Creates reduced versions of routable tiles (OpenStreetMap).")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("area")
                .short('a')
                .long("area")
                .value_name("london|belgium|dummy|pyrenees")
                .about("Sets the bounding box")
                .possible_values(&["belgium", "dummy", "london", "pyrenees"])
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("zoom")
                .short('z')
                .long("zoom")
                .about("Sets the zoom level")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("input")
                .short('i')
                .long("input_dir")
                .about("Root directory to process of input files")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short('o')
                .long("output_dir")
                .about("Root directory to write results to")
                .required(true)
                .takes_value(true),
        )
        .subcommand(
            App::new("reduce_profile")
                .about("Only retain tags that are relevant for the given profile")
                .arg(
                    Arg::with_name("profile")
                        .short('p')
                        .long("profile")
                        .value_name("car|bicycle|pedestrian")
                        .about("Sets the profile to use")
                        .possible_values(&["car", "bicycle", "pedestrian"])
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            App::new("reduce_transit")
                .about("Only retain elements that are necessary to traverse a tile")
                .arg(
                    Arg::with_name("profile")
                        .short('p')
                        .long("profile")
                        .value_name("car|bicycle|pedestrian")
                        .about("Sets the profile to use")
                        .possible_values(&["car", "bicycle", "pedestrian"])
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            App::new("reduce_binary")
                .about("Give up on JSON-LD, and write processed bincode files")
                .arg(
                    Arg::with_name("profile")
                        .short('p')
                        .long("profile")
                        .value_name("car|bicycle|pedestrian")
                        .about("Sets the profile to use")
                        .possible_values(&["car", "bicycle", "pedestrian"])
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            App::new("reduce_indirect_transit")
                .about("Only retain elements that are necessary to traverse a tile")
                .arg(
                    Arg::with_name("profile")
                        .short('p')
                        .long("profile")
                        .value_name("car|bicycle|pedestrian")
                        .about("Sets the profile to use")
                        .possible_values(&["car", "bicycle", "pedestrian"])
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("padding level")
                        .short('l')
                        .long("padding_level")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            App::new("reduce_complete")
                .about("Only retain boundary nodes.")
                .arg(
                    Arg::with_name("profile")
                        .short('p')
                        .long("profile")
                        .value_name("car|bicycle|pedestrian")
                        .about("Sets the profile to use")
                        .possible_values(&["car", "bicycle", "pedestrian"])
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            App::new("reduce_contract")
                .about("Only retain nodes that are relevant for route planning."),
        )
        .subcommand(
            App::new("merge")
                .about("Merge routable tiles into tiles of a higher zoom level"),
        )
        .subcommand(
            App::new("load_tiles")
                .about("Fetches tiles from the given data source and store them locally"),
        )
        .get_matches();

    let zoom = matches
        .value_of("zoom")
        .unwrap()
        .parse::<u32>()
        .expect("Invalid zoom level");

    let area = matches.value_of_t("area").unwrap_or_else(|e| e.exit());
    let [lats, lons] = match area {
        Areas::London => [[51.2424, 51.7334], [-0.5637, 0.3114]], // very dense
        Areas::Belgium => [[49.421, 51.532], [2.4153, 6.5626]], // medium dense
        Areas::Pyrenees => [[41.8872, 43.4263], [-1.9133, 3.3382]], // sparse
        Areas::Dummy => [[51.15, 51.25], [4.40, 4.5]], // tiny piece of belgium
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
            let todo = get_tile_coords(lats, lons, zoom);
            let progress = ProgressBar::new(todo.len() as u64);
            progress.set_style(
                ProgressStyle::default_bar()
                    .template("Pruning Tags [{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} {msg}")
                    .progress_chars("█▓░"),
            );

            todo.par_iter().for_each(|id| {
                let profile_tile_path = get_tile_path(output_dir, id);
                let profile_tile = create_profile_tile(input_dir, id, &profile);
                write_derived_tile(profile_tile, &profile_tile_path).unwrap();
                progress.inc(1);
            });

            progress.finish();
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

            //let index = load_tiles(input_dir, lats, lons, zoom);
            let todo = get_tile_coords(lats, lons, zoom);
            let progress = ProgressBar::new(todo.len() as u64);
            progress.set_style(
                ProgressStyle::default_bar()
                    .template("Pruning Ways [{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} {msg}")
                    .progress_chars("█▓░"),
            );

            todo.par_iter().for_each(|id| {
                let profile_tile_path = get_tile_path(output_dir, id);
                let profile_tile = create_transit_tile(input_dir, id, &profile);
                write_derived_tile(profile_tile, &profile_tile_path).unwrap();
                progress.inc(1);
            });

            progress.finish();
        }
        Some("reduce_indirect_transit") => {
            let profile_name = matches
                .subcommand_matches("reduce_indirect_transit")
                .expect("Subcommand arguments are missing")
                .value_of("profile")
                .unwrap();
            let profile = match profile_name {
                "car" => load_car_profile().unwrap(),
                "pedestrian" => load_pedestrian_profile().unwrap(),
                "bicycle" => load_bicycle_profile().unwrap(),
                _ => unreachable!(),
            };
            let padding_level = matches
                .subcommand_matches("reduce_indirect_transit")
                .expect("Subcommand arguments are missing")
                .value_of("padding level")
                .unwrap()
                .parse::<u32>()
                .expect("Invalid padding level");

            let todo = get_tile_coords(lats, lons, zoom);
            //let todo = vec!(TileCoordinate::new(8345, 5495, 14));
            let progress = ProgressBar::new(todo.len() as u64);
            progress.set_style(
                ProgressStyle::default_bar()
                    .template("Pruning Ways [{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} {msg}")
                    .progress_chars("█▓░"),
            );

            todo.par_iter().for_each(|id| {
                let profile_tile_path = get_tile_path(output_dir, id);
                let profile_tile = create_indirect_transit_tile(
                    input_dir,
                    padding_level,
                    id,
                    &profile,
                );
                write_derived_tile(profile_tile, &profile_tile_path).unwrap();
                progress.inc(1);
            });

            progress.finish();
        }
        Some("reduce_contract") => {
            let todo = get_tile_coords(lats, lons, zoom);
            let progress = ProgressBar::new(todo.len() as u64);
            progress.set_style(
                ProgressStyle::default_bar()
                    .template("Contracting ways [{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} {msg}")
                    .progress_chars("█▓░"),
            );

            todo.par_iter().for_each(|id| {
                let contracted_tile_path = get_tile_path(output_dir, id);
                let contracted_tile = create_contracted_tile(input_dir, id);
                write_derived_tile(contracted_tile, &contracted_tile_path).unwrap();
                progress.inc(1);
            });

            progress.finish();
        }
        Some("merge") => {
            let todo = get_tile_coords(lats, lons, zoom);
            let progress = ProgressBar::new(todo.len() as u64);
            progress.set_style(
                ProgressStyle::default_bar()
                    .template("Merging tiles [{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} {msg}")
                    .progress_chars("█▓░"),
            );

            todo.par_iter().for_each(|id| {
                let merged_tile_path = get_tile_path(output_dir, id);
                let c = id.get_children();
                let merged_tile = create_merged_tile(input_dir, &c, id);
                write_derived_tile(merged_tile, &merged_tile_path).unwrap();
                progress.inc(1);
            });

            progress.finish();
        }
        Some("reduce_binary") => {
            let profile_name = matches
                .subcommand_matches("reduce_binary")
                .expect("Subcommand arguments are missing")
                .value_of("profile")
                .unwrap();
            let profile = match profile_name {
                "car" => load_car_profile().unwrap(),
                "pedestrian" => load_pedestrian_profile().unwrap(),
                "bicycle" => load_bicycle_profile().unwrap(),
                _ => unreachable!(),
            };

            let todo = get_tile_coords(lats, lons, zoom);
            let progress = ProgressBar::new(todo.len() as u64);
            progress.set_style(
                ProgressStyle::default_bar()
                    .template("Creating binary files [{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} {msg}")
                    .progress_chars("█▓░"),
            );

            todo.par_iter().for_each(|id| {
                let merged_tile_path = get_binary_tile_path(output_dir, id);
                let binary_tile = create_binary_tile(input_dir, id, &profile);
                write_flexbuffers_tile(binary_tile, &merged_tile_path);
                progress.inc(1);
            });

            progress.finish();
        }
        Some("load_tiles") => {
            let todo = get_tile_coords(lats, lons, zoom);
            let progress = ProgressBar::new(todo.len() as u64);
            progress.set_style(
                ProgressStyle::default_bar()
                    .template("Loading tiles [{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} {msg}")
                    .progress_chars("█▓░"),
            );

            let failed: Vec<TileCoordinate> = todo.par_iter().filter_map(|id| {
                if let Ok(()) = fetch_tile(input_dir, output_dir, id) {
                    progress.inc(1);
                    return None
                }
                
               Some(*id)
            }).collect();

            eprintln!("Failed to get {} tiles\nThis might be ok, some tiles don't exist", failed.len());

            progress.finish();
        }
        _ => unreachable!(),
    };
}
