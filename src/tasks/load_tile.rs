use crate::{io::{tiles::{write_derived_tile, parse_tile}, get_tile_path}, entities::{tile::DerivedTile, tile_coord::TileCoordinate}};
use anyhow::{Result};
use std::path::Path;

pub fn fetch_tile(
    data_source: &str,
    target_dir: &str,
    coord: &TileCoordinate,
) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .gzip(true)
        .build()?;

    let uri = format!("{}/{}/{}/{}", data_source, coord.zoom, coord.x, coord.y);
    let target_path = get_tile_path(target_dir, coord);
    if !Path::new(&target_path).exists() {
        let response = client.get(&uri).send()?;
        let content =  response.text()?;
        let tile = parse_tile(coord, content)?;
        let derived_tile = DerivedTile::new(*coord, tile.nodes, tile.ways);
        write_derived_tile(derived_tile, &target_path).unwrap();
    }
    
    Ok(())
}