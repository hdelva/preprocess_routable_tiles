pub mod haversine;

use crate::entities::tile_coord::TileCoordinate;

pub fn num2deg(x: u32, y: u32, zoom: u32) -> [f64; 2] {
    let n = 2f64.powf(zoom.into());
    let lon: f64 = f64::from(x) / n * 360.0 - 180.0;
    let pi = std::f64::consts::PI;
    let wat: f64 = pi * (1f64 - 2f64 * f64::from(y) / n);
    let lat_rad = wat.sinh().atan();
    let lat_deg = lat_rad.to_degrees();
    return [lat_deg, lon];
}

pub fn deg2num(lat: f64, lon: f64, zoom: u32) -> TileCoordinate {
    let lat_rad = lat.to_radians();
    let n = 2f64.powf(zoom.into());
    let x = n * ((lon + 180.) / 360.);
    let e = std::f64::consts::E;
    let pi = std::f64::consts::PI;
    let y = n * (1.0 - (lat_rad.tan() + (1. / lat_rad.cos())).log(e) / pi) / 2.0;

    TileCoordinate::new(x.floor() as u32, y.floor() as u32, zoom)
}

pub fn get_tile_edges(coords: &TileCoordinate) -> [f64; 4] {
    let [north, west] = num2deg(coords.x, coords.y, coords.zoom);
    let [south, east] = num2deg(coords.x + 1, coords.y + 1, coords.zoom);
    return [east, north, west, south];
}

pub fn get_tile_coords(lats: [f64; 2], lons: [f64; 2], zoom: u32) -> Vec<TileCoordinate> {
    let [min_lat, max_lat] = lats;
    let [min_lon, max_lon] = lons;

    let top_left = deg2num(max_lat, min_lon, zoom);
    let bottom_right = deg2num(min_lat, max_lon, zoom);

    let mut result = vec!();

    for x in top_left.x .. bottom_right.x + 1 {
        for y in top_left.y .. bottom_right.y + 1 {
            result.push(TileCoordinate::new(x, y, zoom));
        }
    }
    return result;
}