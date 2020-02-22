use crate::entities::node::Node;

pub fn haversine(lat1: f64, lat2: f64, lon1: f64, lon2: f64) -> f64 {
    let r = 6371.00;
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let lon1_rad = lon1.to_radians();
    let lon2_rad = lon2.to_radians();

    let a = ( (lat2_rad - lat1_rad) / 2.00).sin().powf(2.00) +
        lat1_rad.cos() * lat2_rad.cos() * ( (lon2_rad - lon1_rad) / 2.00).sin().powf(2.00);
    let c = 2.00 * ((a).sqrt().atan2((1.00 - a).sqrt()));
    r * c
}

pub fn get_distance(from: &Node, to: &Node) -> f64 {
    // distance in km
    haversine(from.get_lat(), to.get_lat(),from.get_long(), to.get_long())
}
