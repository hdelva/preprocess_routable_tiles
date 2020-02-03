use crate::entities::tile_coord::TileCoordinate;
use std::fs;

pub mod tiles;
pub mod profile;

pub fn get_car_profile_path() -> &'static str {
    return "./car.jsonld";
}

pub fn get_pedestrian_profile_path() -> &'static str {
    return "./pedestrian.jsonld";
}

pub fn get_tile_path(tile: &TileCoordinate) -> String {
    format!("../tiles/{}/{}/{}.json", tile.zoom, tile.x, tile.y)
}

pub fn get_transit_tile_path(profile_name: &str, tile: &TileCoordinate) -> String {
    let main_dir = format!("../tiles/{}/tr_{}/", profile_name, tile.zoom);
    if !fs::metadata(&main_dir).is_ok() {
        fs::create_dir(&main_dir).expect("Could not create directory");
    }

    let sub_dir = format!("{}/{}", main_dir, tile.x);
    if !fs::metadata(&sub_dir).is_ok() {
        fs::create_dir(&sub_dir).expect("Could not create directory");
    }

    format!("{}/{}.json", sub_dir, tile.y)
}

pub fn get_profile_tile_path(profile_name: &str, tile: &TileCoordinate) -> String {
    let main_dir = format!("../tiles/{}/{}/", profile_name, tile.zoom);
    if !fs::metadata(&main_dir).is_ok() {
        fs::create_dir(&main_dir).expect("Could not create directory");
    }

    let sub_dir = format!("{}/{}", main_dir, tile.x);
    if !fs::metadata(&sub_dir).is_ok() {
        fs::create_dir(&sub_dir).expect("Could not create directory");
    }

    format!("{}/{}.json", sub_dir, tile.y)
}