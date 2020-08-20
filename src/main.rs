#![recursion_limit = "128"]

extern crate clap;
#[macro_use] extern crate cached;

use crate::tasks::load_tile::fetch_tile;
use crate::io::tiles::write_flexbuffers_tile;
use crate::tasks::reduce_binary::create_binary_tile;
use crate::io::get_binary_tile_path;
use crate::io::profile::load_bicycle_profile;
use crate::tasks::merge_tiles::create_merged_tile;
use crate::tasks::reduce_profile::create_profile_tile;
use crate::tasks::reduce_transit::create_indirect_transit_tile;
use crate::tasks::reduce_transit::create_transit_tile;
use clap::{App, load_yaml};

mod entities;
mod io;
mod tasks;
mod util;
mod cli;

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
use entities::tile_coord::TileCoordinate;
use cli::{area::Areas, profile::Profiles};

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

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

    let (sub_name, sub_matches) = matches.subcommand();
    let profile = sub_matches
        .and_then(|m| {
            let a = m.value_of_t("profile");
            match a {
                Ok(Profiles::Car) => Some(load_car_profile().unwrap()),
                Ok(Profiles::Bicycle) => Some(load_bicycle_profile().unwrap()),
                Ok(Profiles::Pedestrian) => Some(load_pedestrian_profile().unwrap()),
                Err(_) => None,
            }
        });

    let padding_level = sub_matches
        .and_then(|m| m .value_of("padding"))
        .map(|v| v.parse::<u32>().expect("Invalid padding zoom level"));

    match sub_name {
        "reduce_profile" => {
            let profile = profile.unwrap();

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
        "reduce_transit" => {
            let profile = profile.unwrap();

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
        "reduce_padded_transit" => {
            let profile = profile.unwrap();
            let padding_level = padding_level.unwrap();

            let todo = get_tile_coords(lats, lons, zoom);
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
        "reduce_binary" => {
            let profile = profile.unwrap();

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
        "merge" => {
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
        "fetch_tiles" => {
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
        },
        _ => unreachable!(),
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use io::tiles::load_tile;

    #[test]
    fn test_parse() {
        let coord = TileCoordinate::new(8345, 5495, 14);
        let tile = load_tile(&coord, "./test_data").unwrap();
        assert_eq!(tile.get_nodes().len(), 725);
        assert_eq!(tile.get_ways().len(), 176);
    }

    #[test]
    fn test_profile() {
        let car_profile = load_car_profile().unwrap();
        let coord = TileCoordinate::new(8345, 5495, 14);
        let tile = create_profile_tile("./test_data", &coord, &car_profile);
        assert_eq!(tile.get_nodes().len(), 669);
        assert_eq!(tile.get_ways().len(), 157);

        let bike_profile = load_bicycle_profile().unwrap();
        let coord = TileCoordinate::new(8345, 5495, 14);
        let tile = create_profile_tile("./test_data", &coord, &bike_profile);
        assert_eq!(tile.get_nodes().len(), 630);
        assert_eq!(tile.get_ways().len(), 141);
    }

    #[test]
    fn test_transit() {
        let profile = load_car_profile().unwrap();
        let coord = TileCoordinate::new(8345, 5495, 14);
        let tile = create_transit_tile("./test_data", &coord, &profile);
        assert_eq!(tile.get_nodes().len(), 470);
        assert_eq!(tile.get_ways().len(), 109);
    }

    #[test]
    fn test_padded_transit() {
        let profile = load_car_profile().unwrap();
        let coord = TileCoordinate::new(8345, 5495, 14);
        let tile = create_indirect_transit_tile("./test_data/", 14, &coord, &profile);
        assert_eq!(tile.get_nodes().len(), 307);
        assert_eq!(tile.get_ways().len(), 82);
    }
}