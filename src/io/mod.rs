use crate::entities::tile_coord::TileCoordinate;
use std::fs;

pub mod profile;
pub mod tiles;

pub fn get_car_profile_path() -> &'static str {
    "./car.jsonld"
}

pub fn get_pedestrian_profile_path() -> &'static str {
    "./pedestrian.jsonld"
}

pub fn get_bicycle_profile_path() -> &'static str {
    "./bicycle.jsonld"
}

pub fn get_tile_path(root: &str, tile: &TileCoordinate) -> String {
    let dir = format!("{}/{}/{}", root, tile.zoom, tile.x);
    if fs::metadata(&dir).is_err() {
        fs::create_dir_all(&dir).ok();
    }

    format!("{}/{}.json", dir, tile.y)
}

pub fn get_binary_tile_path(root: &str, tile: &TileCoordinate) -> String {
    let dir = format!("{}/{}/{}", root, tile.zoom, tile.x);
    if fs::metadata(&dir).is_err() {
        fs::create_dir_all(&dir).ok();
    }

    format!("{}/{}.bin", dir, tile.y)
}
