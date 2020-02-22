use crate::entities::node::Node;
use crate::entities::profile::Profile;
use crate::entities::segment::Segment;
use crate::entities::tile::DerivedTile;
use crate::entities::tile::Tile;
use crate::entities::tile_coord::TileCoordinate;
use crate::entities::way::Way;
use std::collections::BTreeMap;

pub fn create_profile_tile<'a>(
    index: &'a BTreeMap<TileCoordinate, Tile>,
    coord: &'a TileCoordinate,
    profile: &Profile,
) -> DerivedTile {
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
    return DerivedTile::new(tile.get_coordinate().clone(), reduced_nodes, reduced_ways);
}
