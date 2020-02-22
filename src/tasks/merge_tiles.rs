use crate::entities::node::Node;
use crate::entities::tile::DerivedTile;
use crate::entities::tile::Tile;
use crate::entities::tile_coord::TileCoordinate;
use crate::entities::way::Way;
use crate::util::deg2num;
use std::collections::btree_map::Entry::Occupied;
use std::collections::btree_map::Entry::Vacant;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

struct WayProxy {
    chains: BTreeMap<String, String>,
    first_candidates: BTreeSet<String>,
    not_first: BTreeSet<String>,
}

impl WayProxy {
    fn new() -> WayProxy {
        return WayProxy {
            chains: BTreeMap::new(),
            first_candidates: BTreeSet::new(),
            not_first: BTreeSet::new(),
        };
    }

    fn add_way(&mut self, way: &Way) {
        for segment in way.get_segments() {
            self.chains.insert(segment.from.to_string(), segment.to.to_string());
            self.first_candidates.insert(segment.from.to_string());
            self.not_first.insert(segment.to.to_string());
        }
    }

    fn check_integrity(&self) -> Result<(), Vec<&String>> {
        let intersection: Vec<&String> =
            self.first_candidates.difference(&self.not_first).collect();
        if intersection.len() > 1 {
            return Err(self.first_candidates.union(&self.not_first).collect());
        }

        return Ok(());
    }

    fn get_node_ids(&mut self) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        let intersection: Vec<&String> =
            self.first_candidates.difference(&self.not_first).collect();
        let mut current_element;
        if intersection.len() == 0 {
            let temp: Vec<&String> = self.first_candidates.iter().collect();
            if temp.is_empty() {
                // should never happen, but it still does
                return Vec::new();
            }
            current_element = temp[0].to_string();
        } else {
            // only follow one chain
            // will break ways around the edges, but should be fine for now
            current_element = intersection[0].to_string();
        }
        result.push(current_element.clone());
        while let Some(next) = self.chains.remove(&current_element) {
            result.push(next.clone());
            current_element = next;
        }
        return result;
    }
}

pub fn create_merged_tile<'a>(
    index: &'a BTreeMap<TileCoordinate, Tile>,
    source_coords: &'a [TileCoordinate],
    target_coord: &'a TileCoordinate,
) -> DerivedTile {
    let mut way_proxies: BTreeMap<String, WayProxy> = BTreeMap::new();
    let mut way_examples: BTreeMap<String, &'a Way> = BTreeMap::new();
    let mut all_nodes: BTreeMap<String, Node> = BTreeMap::new();

    for source_coord in source_coords {
        if let Some(tile) = index.get(source_coord) {
            for (node_id, node) in tile.get_nodes() {
                all_nodes.insert(node_id.to_string(), node.clone());
            }
            for (way_id, way) in tile.get_ways() {
                way_examples.insert(way_id.clone(), way);
                match way_proxies.entry(way_id.clone()) {
                    Vacant(entry) => {
                        let mut new_proxy = WayProxy::new();
                        new_proxy.add_way(way);
                        entry.insert(new_proxy);
                    }
                    Occupied(entry) => {
                        entry.into_mut().add_way(way);
                    }
                }
            }
        }
    }

    for (way_id, proxy) in way_proxies.iter_mut() {
        let mut processed_tiles = BTreeSet::new();
        for source_coord in source_coords {
            processed_tiles.insert(source_coord.clone());
        }
        while let Err(nodes) = proxy.check_integrity() {
            let mut tile_coords = BTreeSet::new();
            for node_id in nodes {
                let node = all_nodes.get(node_id).unwrap();
                let coord = deg2num(node.get_lat(), node.get_long(), source_coords[0].zoom);
                if !processed_tiles.contains(&coord) {
                    tile_coords.insert(coord);
                }
            }
            if tile_coords.len() == 0 {
                break;
            }
            for candidate_coord in tile_coords {
                if let Some(tile) = index.get(&candidate_coord) {
                    let way = tile
                        .get_ways()
                        .get(way_id)
                        .expect("Tile doesn't contain way?");
                    proxy.add_way(way);
                    for node_id in way.get_nodes() {
                        let node = tile
                            .get_nodes()
                            .get(node_id)
                            .expect("Tile doesn't contain node?");
                        all_nodes.insert(node_id.to_string(), node.clone());
                    }
                }
                processed_tiles.insert(candidate_coord);
            }
        }
    }

    let mut all_ways: BTreeMap<String, Way> = BTreeMap::new();
    for (way_id, mut proxy) in way_proxies.into_iter() {
        let nodes = proxy.get_node_ids();
        if !nodes.is_empty() {
            let example_way = way_examples.get(&way_id).unwrap();
            let way = Way::new(
                way_id.clone(),
                nodes,
                None,
                example_way.get_max_speed().clone(),
                example_way.get_tags().clone(),
                example_way.get_undefined_tags().to_vec(),
            );
            all_ways.insert(way_id, way);
        };
    }

    return DerivedTile::new(target_coord.clone(), all_nodes, all_ways);
}
