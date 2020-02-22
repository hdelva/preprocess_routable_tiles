use crate::entities::tile_coord::TileCoordinate;
use std::fs;

pub mod profile;
pub mod tiles;

pub fn get_car_profile_path() -> &'static str {
    return "./car.jsonld";
}

pub fn get_pedestrian_profile_path() -> &'static str {
    return "./pedestrian.jsonld";
}

pub fn get_bicycle_profile_path() -> &'static str {
    return "./bicycle.jsonld";
}

pub fn get_tile_path(root: &str, tile: &TileCoordinate) -> String {
    let dir = format!("{}/{}/{}/", root, tile.zoom, tile.x);
    if !fs::metadata(&dir).is_ok() {
        fs::create_dir_all(&dir).ok();
    }

    format!("{}/{}.json", dir, tile.y)
}
