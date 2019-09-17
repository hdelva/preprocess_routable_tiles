## Disclaimer

This project contains mostly prototype code, but apart from the structure it should be relatively clean. 

## Structure

There are 4 modules of code spread over 2 IPython notebooks, 1 Python script and 1 Rust project. 

The IPython/Python modules are:

* `python_code/fetch_tiles.py`: fetches raw tile from the web;
* `python_code/merge_tiles.ipynb`: merges tiles from one zoom level into tiles of a higher zoom level;
* `python_code/contract_tiles.ipynb`: retains only interesting nodes and creates weighted edges between them.

The main (Rust) project identifies which ways are useful to traverse a tile -- and discards all others. 

## See also

The [planner.js](https://github.com/openplannerteam/planner.js) project uses these tiles to improve query answering times (see `src/planner/road/RoadPlannerPathfindingExperimental.ts`). 

There are related slides for the State of the Map 2019 conference [here](https://hdelva.be/slides/sotm2019/). 