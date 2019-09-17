use crate::entities::tile_coord::TileCoordinate;
use std::fs;

pub mod tiles;
pub mod profile;

pub fn get_profile_path() -> String {
    return "../car.json".to_owned();
}

pub fn get_tile_path(tile: &TileCoordinate) -> String {
    format!("../tiles/{}/{}/{}.json", tile.zoom, tile.x, tile.y)
}

pub fn get_transit_tile_path(tile: &TileCoordinate) -> String {
    let main_dir = format!("../tiles/tr_{}/", tile.zoom);
    if !fs::metadata(&main_dir).is_ok() {
        fs::create_dir(&main_dir).expect("Could not create directory");
    }

    let sub_dir = format!("{}/{}", main_dir, tile.x);
    if !fs::metadata(&sub_dir).is_ok() {
        fs::create_dir(&sub_dir).expect("Could not create directory");
    }

    format!("{}/{}.json", sub_dir, tile.y)
}