# Preprocessed Routable Tiles

The Routable Tiles dataset is a republished version of the OpenStreetMap road network, specifically for the use case of route planning. For more information about this data, see https://tiles.openplanner.team/planet/14/8411/5485/ for an example of one tile, or read the paper at http://pieter.pm/demo-paper-routable-tiles/. 

Although this data can be used by anyone (or anything) to answer any route planning query without having to depend on a centralized route planning service, the sheer size of road networks means that finding a route between two points that are just 100 km apart can use several gigabytes of (uncompressed) data. This projects aims to create filtered views of the original data, so that clients can be more selective in the data they process – essentially moving some route planning logic back to the service provider. 

Three different transformations are currently supported:

* Profile-based: Given a vehicle profile in the OpenPlanner Team's vehicle profile vocabulary (currently available at http://hdelva.be/profile/ns/profile.html), discard all ways and nodes that the vehicle has no access to. For example, cars have no access to dedicated cycling paths.
* Transit-based: Not to be confused with public transit, this transformation determines which ways and nodes are necessary to traverse a tile – and discards all others.
* Contraction-based: Not all nodes on a way are relevant for route planning; many of them are there for visualization purposes (e.g., to describe the curvature of the street). These nodes can be discarded, as long as the distances between the remaining nodes are published as well. 

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

A folder containing the required base Routable tiles has to be provided in a folder structured as follows 

```
{zoom}/{tile_x}/{tile_y}.jsonld
```

A helper script (`fetch_tiles.py`) is included that fetches these tiles, although its dependencies still have to manually installed for now. 

---

The executable (which by default gets written to `./target/release`) can be run as follows:

```
USAGE:
    preprocess --area <belgium|dummy> --input_dir <input> --output_dir <output> --zoom <zoom> <SUBCOMMAND> [--profile <car|bicycle|pedestrian>]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --area <belgium|dummy>    Sets the bounding box [possible values: belgium, dummy]
    -i, --input_dir <input>       Root directory to process of input files
    -o, --output_dir <output>     Root directory to write results to
    -z, --zoom <zoom>             Sets the zoom level

SUBCOMMANDS:
    help               Prints this message or the help of the given subcommand(s)
    merge              Merge routable tiles into tiles of a higher zoom level
    reduce_contract    Only retain nodes that
    reduce_profile     Only retain tags that are relevant for the given profile
    reduce_transit     Only retain elements that are necessary to traverse a tile
```

## Examples

**Example 1**: Merging tiles of zoom level 14, to create a tiles of zoom level 13

```
./target/release/preprocess --area belgium --zoom 14 -i ./tiles -o ./tiles merge
```

**Example 2**: Creating zoom level 14 tiles suitable for pedestrians

```
./target/release/preprocess --area belgium --zoom 14 -i ./tiles -o ./tiles/pedestrian reduce_profile --profile pedestrian
```

**Example 3**: Creating zoom level 12 transit tiles for cars

```
./target/release/preprocess --area belgium --zoom 12 -i ./tiles/car -o ./tiles/car/transit reduce_transit --profile car
```

**Example 4**: Contracting unnecessary nodes from existing transit tiles

```
./target/release/preprocess --area belgium --zoom 12 -i ./tiles/car/transit -o ./tiles/car/contracted reduce_contract
```

## See also

This project was presented at the State of the Map 2019 conference, slides are available [here](https://hdelva.be/slides/sotm2019/). 

## License

[MIT](https://choosealicense.com/licenses/mit/)