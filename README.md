# Preprocessed Routable Tiles

The Routable Tiles dataset is a republished version of the OpenStreetMap road network, specifically for the use case of route planning. For more information about this data, see https://tiles.openplanner.team/planet/14/8411/5485/ for an example of one tile, or read the paper at http://pieter.pm/demo-paper-routable-tiles/. 

Although this data can be used by anyone (or anything) to answer any route planning query without having to depend on a centralized route planning service, the sheer size of road networks means that finding a route between two points that are just 100 km apart can use several gigabytes of (uncompressed) data. This projects aims to create filtered views of the original data, so that clients can be more selective in the data they process – essentially moving some route planning logic back to the service provider. 

Three different transformations are currently supported:

* Profile-based: Given a vehicle profile in the OpenPlanner Team's vehicle profile vocabulary (currently available at http://hdelva.be/profile/ns/profile.html), discard all ways and nodes that the vehicle has no access to. For example, cars have no access to dedicated cycling paths.
* Transit-based: Not to be confused with public transit, this transformation determines which ways and nodes are necessary to traverse a tile – and discards all others.
* Binary-based: Store a weighted edge graph as a binary (protobuf) file. The first thing route planners have to do when using the original RDF data is parse it, and align it with a routing profile. This step is costly because RDF data is conceptually free-form, which forces the parsing to be schemaless. In turn, this means that the data has to be parsed into a map (e.g., a HashMap), instead of a compact and efficient `struct`. The resulting edge graph does have a strict structure however, which means it can be stored -- and parsed -- more efficiently. The downside of this approach is that the resulting data only makes sense if the routing profile is fixed as well, as even small changes in the profile can no longer be propagated into the edge graph anymore.

An additional, fourth, transformation is implemented but currently hidden. Not all nodes on a way are relevant for route planning; many of them are there for visualization purposes (e.g., to describe the curvature of the street). These nodes can be discarded, as long as the distances between the remaining nodes are published as well. The resulting data can be used for route planning and even navigation instructions, but cannot be visualized on an existing map anymore as the curvature of the roads is lost. As a result, this transformation does not really meet our requirements of building _reusable_ preprocessed road network data. 

## Installation

This project was written in Rust, and tested with rustc v1.41.0. The Rust toolchain can be installed with [rustup](https://rustup.rs/#). This toolchain includes the [Cargo](https://doc.rust-lang.org/cargo/) package manager, which can be used to build this project and its dependencies.

```
cargo build --release
```

Depending on your CPU's architecture, running this command with an additional compiler flag can have a noticeable impact on performance. 

```
RUSTFLAGS='-C target-cpu=native' cargo build --release
```

## Usage

The executable (which by default gets written to `./target/release`) can be run as follows:

```
USAGE:
    preprocess --area <dummy|london|belgium|pyrenees> --zoom <zoom> --input_dir <input> --output_dir <output> <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --area <dummy|london|belgium|pyrenees>
            Sets the bounding box [possible values: belgium, dummy, london, pyrenees]

    -i, --input_dir <input>                       Root directory to process of input files
    -o, --output_dir <output>                     Root directory to write results to
    -z, --zoom <zoom>                             Sets the zoom level

SUBCOMMANDS:
    fetch_tiles              Fetches tiles from the given data source and store them locally
    help                     Prints this message or the help of the given subcommand(s)
    merge                    Merge routable tiles into tiles of the given zoom level
    reduce_binary            Store a binary encoded edge graph, instead of raw RDF data
    reduce_padded_transit    Only retain elements that are needed to traverse the area around a given tile
    reduce_profile           Only retain tags that are relevant for the given profile
    reduce_transit           Only retain elements that are necessary to traverse a tile
```

## Examples

**Example 1**: Fetch (zoom level 14) tiles from the default Routable Tiles server

```
./target/release/preprocess --area london --zoom 14 -i https://tiles.openplanner.team/planet -o ./tiles fetch_tiles
```

**Example 2**: Merging tiles of zoom level 14, to create a tiles of zoom level 13

```
./target/release/preprocess --area london --zoom 13 -i ./tiles -o ./tiles merge
```

**Example 3**: Creating zoom level 14 tiles suitable for pedestrians

```
./target/release/preprocess --area belgium --zoom 14 -i ./tiles -o ./tiles/pedestrian reduce_profile --profile pedestrian
```

**Example 4**: Creating zoom level 12 transit tiles for cars

```
./target/release/preprocess --area belgium --zoom 12 -i ./tiles/car -o ./tiles/car/transit reduce_transit --profile car
```

**Example 5**: Creating zoom level 14 transit tiles for cars, with a padding layer of level 14 tiles

Concretely, this means that instead of calculating how to move between the tile's edges -- it's going to add a padding layer around that tile, and calculate how to move between the padding layer's edges. 

```
./target/release/preprocess --area belgium --zoom 14 -i ./tiles/car/transit -o ./tiles/car/p_transit reduce_padded_transit --profile car
```

**Example 6**: Store a weighted edge graph as a binary file instead of the raw RDF data.

```
./target/release/preprocess --area belgium --zoom 12 -i ./tiles/car/transit -o ./tiles/car/contracted reduce_contract
```

## See also

This project was presented at the State of the Map 2019 conference, slides are available [here](https://hdelva.be/slides/sotm2019/). 

## License

[MIT](https://choosealicense.com/licenses/mit/)
