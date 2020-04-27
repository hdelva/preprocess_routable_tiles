use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;

use crate::entities::node::Node;
use crate::entities::tile::{DerivedTile, Tile};
use crate::entities::tile_coord::TileCoordinate;
use crate::entities::way::Way;
use crate::util::get_tile_edges;
use std::fs;

#[derive(Debug)]
pub enum Error {
    NotAFile(String),
    InvalidFile(String),
    NotJson,
    MissingID,
    MissingLatitude,
    MissingLongitude,
    MissingNodes,
}

pub fn load_tile(coordinate: TileCoordinate, path: &str) -> Result<Tile, Error> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        _ => return Err(Error::NotAFile(path.to_string())),
    };

    let mut data = String::new();
    if file.read_to_string(&mut data).is_err() {
        return Err(Error::InvalidFile(path.to_string()));
    }

    let v: Value = match serde_json::from_str(&data) {
        Ok(v) => v,
        Err(_) => return Err(Error::NotJson),
    };

    let graph = match v["@graph"].as_array() {
        Some(graph) => graph,
        None => return Err(Error::NotJson),
    };

    let mut nodes = BTreeMap::new();
    let mut ways = BTreeMap::new();

    for entity in graph {
        if entity["@type"].as_str().unwrap() == "osm:Node" {
            match create_node(entity) {
                Ok(node) => nodes.insert(node.get_id().to_string(), node),
                _ => None,
            };
        } else if entity["@type"].as_str().unwrap() == "osm:Way" {
            match create_way(entity) {
                Ok(way) => ways.insert(way.get_id().to_string(), way),
                _ => None,
            };
        }
    }

    Ok(Tile::new(coordinate, nodes, ways))
}

fn create_node(entity: &Value) -> Result<Node, Error> {
    let id = match entity["@id"].as_str() {
        Some(id) => id.to_owned(),
        _ => return Err(Error::MissingID),
    };

    let lat = match entity["geo:lat"].as_f64() {
        Some(id) => id,
        _ => return Err(Error::MissingLatitude),
    };

    let long = match entity["geo:long"].as_f64() {
        Some(id) => id,
        _ => return Err(Error::MissingLongitude),
    };

    let mut tags = BTreeMap::new();
    for (key, value) in entity.as_object().unwrap() {
        if key != "osm:hasNodes" && key.starts_with("osm:") {
            if value.is_string() {
                tags.insert(key.to_string(), value.as_str().unwrap().to_owned());
            } else if value.is_array() {
                // probably hasTag
            } else {
                println!("{} {}", key, value);
            }
        }
    }

    let mut undefined_tags = Vec::new();
    if let Some(values) = entity.get("osm:hasTag") {
        undefined_tags = values
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_owned())
            .collect();
    }

    Ok(Node::new(id, lat, long, tags, undefined_tags))
}

fn create_way(entity: &Value) -> Result<Way, Error> {
    let id = match entity["@id"].as_str() {
        Some(id) => id.to_owned(),
        _ => return Err(Error::MissingID),
    };

    let nodes: Vec<String> = match entity["osm:hasNodes"].as_array() {
        Some(nodes) => nodes
            .iter()
            .map(|id| id.as_str().unwrap().to_string())
            .collect(),
        _ => return Err(Error::MissingNodes),
    };

    let mut tags = BTreeMap::new();
    for (key, value) in entity.as_object().unwrap() {
        if key != "osm:hasNodes" && key.starts_with("osm") {
            if value.is_string() {
                tags.insert(key.to_string(), value.as_str().unwrap().to_owned());
            } else if value.is_array() {
                //
            } else {
                println!("{} {}", key, value);
            }
        }
    }

    let mut undefined_tags = Vec::new();
    if let Some(values) = entity.get("osm:hasTag") {
        undefined_tags = values
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_owned())
            .collect();
    }

    let mut max_speed = None;
    if let Some(broken_string_value) = tags.get("osm:maxspeed") {
        let float_value: f64 = broken_string_value.parse().unwrap();
        max_speed = Some(float_value);
    }

    Ok(Way::new(id, nodes, None, max_speed, tags, undefined_tags))
}

pub fn write_derived_tile(tile: DerivedTile, path: &str) {
    let mut graph: Vec<Value> = tile
        .get_nodes()
        .values()
        .map(|node| {
            let mut blob = BTreeMap::new();
            blob.insert("@type".to_owned(), json!("osm:Node"));
            blob.insert("@id".to_owned(), json!(node.get_id()));
            blob.insert("geo:long".to_owned(), json!(node.get_long()));
            blob.insert("geo:lat".to_owned(), json!(node.get_lat()));

            if !node.get_undefined_tags().is_empty() {
                blob.insert("osm:hasTag".to_owned(), json!(node.get_undefined_tags()));
            }

            for (key, value) in node.get_tags() {
                blob.insert(key.to_string(), json!(value));
            }

            json!(blob)
        })
        .collect();

    let mut ways: Vec<Value> = tile
        .get_ways()
        .values()
        .map(|way| {
            let mut blob = BTreeMap::new();
            blob.insert("@type".to_owned(), json!("osm:Way"));
            blob.insert("@id".to_owned(), json!(way.get_id()));
            if let Some(weights) = way.get_distances() {
                let mut edges = BTreeMap::new();
                edges.insert("osm:hasNodes".to_owned(), json!(way.get_nodes()));
                edges.insert("osm:hasWeights".to_owned(), json!(weights));
                blob.insert("osm:hasEdges".to_owned(), json!(edges));
            } else {
                blob.insert("osm:hasNodes".to_owned(), json!(way.get_nodes()));
            }

            if !way.get_undefined_tags().is_empty() {
                blob.insert("osm:hasTag".to_owned(), json!(way.get_undefined_tags()));
            }

            for (key, value) in way.get_tags() {
                blob.insert(key.to_string(), json!(value));
            }

            json!(blob)
        })
        .collect();

    graph.append(&mut ways);
    let context = json!({
    "tiles":"https://w3id.org/tree/terms#",
    "hydra":"http://www.w3.org/ns/hydra/core#",
    "osm":"https://w3id.org/openstreetmap/terms#",
    "rdfs":"http://www.w3.org/2000/01/rdf-schema#",
    "geo":"http://www.w3.org/2003/01/geo/wgs84_pos#",
    "dcterms":"http://purl.org/dc/terms/",
    "dcterms:license":{"@type":"@id"},
    "hydra:variableRepresentation":{"@type":"@id"},
    "hydra:property":{"@type":"@id"},
    "osm:access":{"@type":"@id"},
    "osm:barrier":{"@type":"@id"},
    "osm:bicycle":{"@type":"@id"},
    "osm:construction":{"@type":"@id"},
    "osm:crossing":{"@type":"@id"},
    "osm:cycleway":{"@type":"@id"},
    "osm:footway":{"@type":"@id"},
    "osm:highway":{"@type":"@id"},
    "osm:motor_vehicle":{"@type":"@id"},
    "osm:motorcar":{"@type":"@id"},
    "osm:oneway_bicycle":{"@type":"@id"},
    "osm:oneway":{"@type":"@id"},
    "osm:smoothness":{"@type":"@id"},
    "osm:surface":{"@type":"@id"},
    "osm:tracktype":{"@type":"@id"},
    "osm:vehicle":{"@type":"@id"},
    "osm:hasNodes":{"@container":"@list","@type":"@id"},
    "osm:hasMembers":{"@container":"@list","@type":"@id"}}
    );

    let file = json!({
        "@context": context,
        "@id":"https://tiles.openplanner.team/planet/14/8411/5485/",
        "tiles:zoom":tile.get_coordinate().zoom,
        "tiles:longitudeTile":tile.get_coordinate().x,
        "tiles:latitudeTile":tile.get_coordinate().y,
        "dcterms:isPartOf":{
            "@id":"https://tiles.openplanner.team/planet/",
            "@type":"hydra:Collection",
            "dcterms:license":"http://opendatacommons.org/licenses/odbl/1-0/",
            "dcterms:rights":"http://www.openstreetmap.org/copyright",
            "hydra:search":{
                "@type":"hydra:IriTemplate",
                "hydra:template":"https://tiles.openplanner.team/planet/14/{x}/{y}",
                "hydra:variableRepresentation":"hydra:BasicRepresentation",
                "hydra:mapping":[{
                    "@type":"hydra:IriTemplateMapping",
                    "hydra:variable":"x",
                    "hydra:property":"tiles:longitudeTile",
                    "hydra:required":true
                },{
                    "@type":"hydra:IriTemplateMapping",
                    "hydra:variable":"y",
                    "hydra:property":"tiles:latitudeTile",
                    "hydra:required":true
                }]
            }
        },
        "@graph": graph
    });

    fs::write(path, file.to_string()).expect("Unable to write file");
}

//this writes transit tiles for level between [8, 14[
pub fn write_derived_tile_wkt_tree(tile: DerivedTile, path: &str) {
    let mut graph: Vec<Value> = tile
        .get_nodes()
        .values()
        .map(|node| {
            let mut blob = BTreeMap::new();
            blob.insert("@type".to_owned(), json!("osm:Node"));
            blob.insert("@id".to_owned(), json!(node.get_id()));
            blob.insert(
                "geo:asWKT".to_owned(),
                json!(format!(
                    "<http://www.opengis.net/def/crs/OGC/1.3/CRS84> POINT({} {})",
                    node.get_long(),
                    node.get_lat()
                )),
            );

            if !node.get_undefined_tags().is_empty() {
                blob.insert("osm:hasTag".to_owned(), json!(node.get_undefined_tags()));
            }

            for (key, value) in node.get_tags() {
                blob.insert(key.to_string(), json!(value));
            }

            json!(blob)
        })
        .collect();

    let mut ways: Vec<Value> = tile
        .get_ways()
        .values()
        .map(|way| {
            let mut blob = BTreeMap::new();
            blob.insert("@type".to_owned(), json!("osm:Way"));
            blob.insert("@id".to_owned(), json!(way.get_id()));
            if let Some(weights) = way.get_distances() {
                let mut edges = BTreeMap::new();
                edges.insert("osm:hasNodes".to_owned(), json!(way.get_nodes()));
                edges.insert("osm:hasWeights".to_owned(), json!(weights));
                blob.insert("osm:hasEdges".to_owned(), json!(edges));
            } else {
                blob.insert("osm:hasNodes".to_owned(), json!(way.get_nodes()));
            }

            if !way.get_undefined_tags().is_empty() {
                blob.insert("osm:hasTag".to_owned(), json!(way.get_undefined_tags()));
            }

            for (key, value) in way.get_tags() {
                blob.insert(key.to_string(), json!(value));
            }

            json!(blob)
        })
        .collect();


    let [ upper_left_child, upper_right_child, down_left_child, down_right_child ] = &tile.get_coordinate().get_children();

    let [ east_upper_left_child, north_upper_left_child, west_upper_left_child, south_upper_left_child ] = get_tile_edges(&upper_left_child);
    let [ east_upper_right_child, north_upper_right_child, west_upper_right_child, south_upper_right_child ] = get_tile_edges(&upper_right_child);
    let [ east_down_left_child, north_down_left_child, west_down_left_child, south_down_left_child ] = get_tile_edges(&down_left_child);
    let [ east_down_right_child, north_down_right_child, west_down_right_child, south_down_right_child ] = get_tile_edges(&down_right_child);

    graph.append(&mut ways);
    let context = json!({
    "tiles":"https://w3id.org/tree/terms#",
    "hydra":"http://www.w3.org/ns/hydra/core#",
    "osm":"https://w3id.org/openstreetmap/terms#",
    "rdfs":"http://www.w3.org/2000/01/rdf-schema#",
    "geo":"http://www.opengis.net/ont/geosparql#",
    "geo:asWKT":{
        "@type":"geo:wktLiteral"
    },
    "tree": "https://w3id.org/tree#",
    "dcterms":"http://purl.org/dc/terms/",
    "dcterms:license":{"@type":"@id"},
    "hydra:variableRepresentation":{"@type":"@id"},
    "hydra:property":{"@type":"@id"},
    "osm:access":{"@type":"@id"},
    "osm:barrier":{"@type":"@id"},
    "osm:bicycle":{"@type":"@id"},
    "osm:construction":{"@type":"@id"},
    "osm:crossing":{"@type":"@id"},
    "osm:cycleway":{"@type":"@id"},
    "osm:footway":{"@type":"@id"},
    "osm:highway":{"@type":"@id"},
    "osm:motor_vehicle":{"@type":"@id"},
    "osm:motorcar":{"@type":"@id"},
    "osm:oneway_bicycle":{"@type":"@id"},
    "osm:oneway":{"@type":"@id"},
    "osm:smoothness":{"@type":"@id"},
    "osm:surface":{"@type":"@id"},
    "osm:tracktype":{"@type":"@id"},
    "osm:vehicle":{"@type":"@id"},
    "osm:hasNodes":{"@container":"@list","@type":"@id"},
    "osm:hasMembers":{"@container":"@list","@type":"@id"},
    "tree:node":{"@type":"@id"},
    "tree:path":{"@type":"@id"}}
    );

    let file = json!({
        "@context": context,
        "@id":format!("https://example.transitTree.org/root/{}/{}/{}/", tile.get_coordinate().zoom, tile.get_coordinate().x, tile.get_coordinate().y),
        "tiles:zoom":tile.get_coordinate().zoom,
        "tiles:longitudeTile":tile.get_coordinate().x,
        "tiles:latitudeTile":tile.get_coordinate().y,
        "tree:relation": [
            {
                "@type": "tree:GeospatiallyContainsRelation",
                "tree:node": format!("https://example.transitTree.org/root/{}/{}/{}", tile.get_coordinate().zoom +1, tile.get_coordinate().x*2, tile.get_coordinate().y*2),
                "tree:path": "geo:asWKT",
                "tree:value": format!("POLYGON(({} {}, {} {}, {} {}, {} {}, {} {}))", west_upper_left_child, north_upper_left_child, east_upper_left_child, north_upper_left_child, east_upper_left_child, south_upper_left_child, west_upper_left_child, south_upper_left_child, west_upper_left_child, north_upper_left_child)
            },
            {
                "@type": "tree:GeospatiallyContainsRelation",
                "tree:node": format!("https://example.transitTree.org/root/{}/{}/{}", tile.get_coordinate().zoom +1, tile.get_coordinate().x*2 +1, tile.get_coordinate().y*2),
                "tree:path": "geo:asWKT",
                "tree:value": format!("POLYGON(({} {}, {} {}, {} {}, {} {}, {} {}))", west_upper_right_child, north_upper_right_child, east_upper_right_child, north_upper_right_child, east_upper_right_child, south_upper_right_child, west_upper_right_child, south_upper_right_child, west_upper_right_child, north_upper_right_child)
            },
            {
                "@type": "tree:GeospatiallyContainsRelation",
                "tree:node": format!("https://example.transitTree.org/root/{}/{}/{}", tile.get_coordinate().zoom +1, tile.get_coordinate().x*2, tile.get_coordinate().y*2 +1),
                "tree:path": "geo:asWKT",
                "tree:value": format!("POLYGON(({} {}, {} {}, {} {}, {} {}, {} {}))", west_down_left_child, north_down_left_child, east_down_left_child, north_down_left_child, east_down_left_child, south_down_left_child, west_down_left_child, south_down_left_child, west_down_left_child, north_down_left_child)
            },
            {
                "@type": "tree:GeospatiallyContainsRelation",
                "tree:node": format!("https://example.transitTree.org/root/{}/{}/{}", tile.get_coordinate().zoom +1, tile.get_coordinate().x*2 +1, tile.get_coordinate().y*2 +1),
                "tree:path": "geo:asWKT",
                "tree:value": format!("POLYGON(({} {}, {} {}, {} {}, {} {}, {} {}))", west_down_right_child, north_down_right_child, east_down_right_child, north_down_right_child, east_down_right_child, south_down_right_child, west_down_right_child, south_down_right_child, west_down_right_child, north_down_right_child)
            }
           ],
        "dcterms:isPartOf":{
            "@id":"https://example.transitTree.org/root",
            "@type":"hydra:Collection",
            "dcterms:license":"http://opendatacommons.org/licenses/odbl/1-0/",
            "dcterms:rights":"http://www.openstreetmap.org/copyright",
        },
        "@graph": graph
    });

    fs::write(path, file.to_string()).expect("Unable to write file");
}
