# Ship Routing

A submission for the "Algortihms on OSM-Data" lecture that enables users to perform ship routing, aka routing on sea.

## Usage

To build and run the application, the following commands can be used:

### Native

Requirements: `make`, `rustc` >= 1.70, `cc`

- `make convert`: Convert the given PBF file into a planet geojson file by extracting and closing the coastlines.
- `make network`: Takes the planet geojson file and generates a given number of nodes located on water and creates a geojson file containing the generated nodes and a FMI file containing the graph representing the node neighbors.
- `make server`: This command starts a local webserver that can be accessed on [localhost:3030](http://localhost:3030)

### Container

Requirements: `docker`, `podman` or similar.

The container can be built locally by running `docker build . -f Containerfile`. To run the prebuilt image use `docker run --init --rm -p 3030:3030 ghcr.io/christianstaib/ship_routing`.
