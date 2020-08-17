use std::collections::BTreeSet;
use crate::entities::tile::Tile;

pub fn get_edge_nodes(tile: &Tile, bounds: [f64; 4]) -> BTreeSet<String> {
    let [e, n, w, s] = bounds;
    let mut oob = BTreeSet::new();
    for node in tile.get_nodes().values() {
        let lat = node.get_lat();
        let long = node.get_long();

        if node.get_id() == "http://www.openstreetmap.org/node/1428406494" {
            eprintln!("oob {}", !(s <= lat && lat <= n && w <= long && long <= e));
        }
        if !(s <= lat && lat <= n && w <= long && long <= e) {
            oob.insert(node.get_id().to_owned());
        }
    }
    oob
}
