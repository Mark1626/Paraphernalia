module.exports = {
    path: "./assets/",
    tilesize: 16,
    tiles:[
        {name: "dirt_road_corner", symmetry: "L"},
        {name: "dirt_road", symmetry: "I"},
        {name: "dirt_road_t", symmetry: "T"},
        {name: "tree_single", symmetry: "X", weight: 0.1},
        {name: "tree_multiple", symmetry: "X", weight: 0.1},
        {name: "building_a", symmetry: "X", weight: 0.01},
        {name: "building_b", symmetry: "X", weight: 0.01},
        {name: "building_c", symmetry: "X", weight: 0.01},
        {name: "building_d", symmetry: "X", weight: 0.01},
        {name: "building_e", symmetry: "X", weight: 0.01},
    ],
    neighbors: [
        {left: "dirt_road", right: "dirt_road_corner"},
        {left: "dirt_road_corner 1", right: "dirt_road_corner"},
        {left: "dirt_road_corner 2", right: "dirt_road_corner"},
        {left: "dirt_road_corner 3", right: "dirt_road_corner"},
        {left: "dirt_road 1", right: "dirt_road 1"},
        {left: "dirt_road_corner", right: "dirt_road 1"},
        {left: "dirt_road 2", right: "dirt_road_corner"},
    ],
}