const { PathManager } = require("./PathManager.ts");
const { record, save } = require("./recorder.cjs");

PathManager.top_node_id = "top_node";

console.log("++++++++++++++++++++++++++++++++++++++++++1 0,10+");

PathManager.path_tree = {
    "top_node": {
        from: [0,0],
        to: [0,0],
        playing: false,
        parent: "-1",
        children: [],
        changes: [
            {
                id: "first_node",
                change: 1
            },
            {
                id: "second_node",
                change: 1
            }],
        moveNull: true
    },
    "first_node": {
        from: [0,0],
        to: [5,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    },
    "second_node": {
        from: [0,0],
        to: [10,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    }
}
console.log(JSON.stringify(PathManager.unfoldCompact(PathManager.getChanges())));

console.log("++++++++++++++++++++++++++++++++++++++++++2 0,2+ & 3,10+ || insg(0,10)+");
PathManager.path_tree = {
    "top_node": {
        from: [0,0],
        to: [0,0],
        playing: false,
        parent: "-1",
        children: [],
        changes: [
            {
                id: "first_node",
                change: 1
            },
            {
                id: "second_node",
                change: 1
            }],
        moveNull: true
    },
    "first_node": {
        from: [0,0],
        to: [5,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    },
    "second_node": {
        from: [3,0],
        to: [10,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    }
}



console.log(JSON.stringify(PathManager.unfoldCompact(PathManager.getChanges())));

console.log("++++++++++++++++++++++++++++++++++++++++++3 0,10+");
PathManager.path_tree = {
    "top_node": {
        from: [0,0],
        to: [0,0],
        playing: false,
        parent: "-1",
        children: [],
        changes: [
            {
                id: "first_node",
                change: -1
            },
            {
                id: "second_node",
                change: 1
            }],
        moveNull: true
    },
    "first_node": {
        from: [0,0],
        to: [5,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    },
    "second_node": {
        from: [0,0],
        to: [10,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    }
}



console.log(JSON.stringify(PathManager.unfoldCompact(PathManager.getChanges())));

console.log("++++++++++++++++++++++++++++++++++++++++++4 0,5~&6,10+");
PathManager.path_tree = {
    "top_node": {
        from: [0,0],
        to: [0,0],
        playing: false,
        parent: "-1",
        children: [],
        changes: [
            {
                id: "first_node",
                change: 0
            },
            {
                id: "second_node",
                change: 1
            }],
        moveNull: true
    },
    "first_node": {
        from: [0,0],
        to: [5,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    },
    "second_node": {
        from: [0,0],
        to: [10,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    }
}







console.log(JSON.stringify(PathManager.unfoldCompact(PathManager.getChanges())));

console.log("++++++++++++++++++++++++++++++++++++++++++5 0,3- & 4,8+ & 9,10- || since new simple path converter -> it needs to start from 0,0");
PathManager.path_tree = {
    "top_node": {
        from: [0,0],
        to: [0,0],
        playing: false,
        parent: "-1",
        children: ["second_node"],
        changes: [
            {
                id: "first_node",
                change: -1
            },
            {
                id: "second_node",
                change: 1
            }],
        moveNull: true
    },
    "first_node": {
        from: [0,0],
        to: [10,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    },
    "second_node": {
        from: [4,0],
        to: [8,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    }
}
//console.log(JSON.stringify(PathManager.unfoldCompact(PathManager.getChanges())));
console.log(PathManager.convertCompact());



console.log("++++++++++++++++++++++++++++++++++++++++++6 0,8~ & 9,10+");
PathManager.path_tree = {
    "top_node": {
        from: [0,0],
        to: [0,0],
        playing: false,
        parent: "-1",
        children: ["first_node", "second_node"],
        changes: [
            {
                id: "first_node",
                change: 1
            },
            {
                id: "second_node",
                change: 0
            }],
        moveNull: true
    },
    "first_node": {
        from: [0,0],
        to: [10,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    },
    "second_node": {
        from: [0,0],
        to: [8,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    }
}
//console.log(JSON.stringify(PathManager.unfoldCompact(PathManager.getChanges())));
console.log(PathManager.convertCompact());




console.log("++++++++++++++++++++++++++++++++++++++++++7 0,10+");
PathManager.path_tree = {
    "top_node": {
        from: [0,0],
        to: [0,0],
        playing: false,
        parent: "-1",
        children: ["first_node", "second_node"],
        changes: [
            {
                id: "first_node",
                change: 1
            },
            {
                id: "second_node",
                change: 1
            }],
        moveNull: true
    },
    "first_node": {
        from: [0,0],
        to: [10,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    },
    "second_node": {
        from: [0,0],
        to: [10,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    }
}
//console.log(JSON.stringify(PathManager.unfoldCompact(PathManager.getChanges())));
console.log(PathManager.convertCompact());



console.log("++++++++++++++++++++++++++++++++++++++++++8 insg(0,10)+");
PathManager.path_tree = {
    "top_node": {
        from: [0,0],
        to: [0,0],
        playing: false,
        parent: "-1",
        children: ["first_node", "second_node", "third_node"],
        changes: [
            {
                id: "first_node",
                change: 1
            },
            {
                id: "second_node",
                change: 1
            },
            {
                id: "third_node",
                change: 1
            }],
        moveNull: true
    },
    "first_node": {
        from: [0,0],
        to: [5,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    },
    "second_node": {
        from: [6,0],
        to: [10,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    },
    "third_node": {
        from: [3,0],
        to: [7,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    }
}
//console.log(JSON.stringify(PathManager.unfoldCompact(PathManager.getChanges())));
console.log(PathManager.convertCompact());



console.log("++++++++++++++++++++++++++++++++++++++++++9 0,5+ & 6,10+");
PathManager.path_tree = {
    "top_node": {
        from: [0,0],
        to: [0,0],
        playing: false,
        parent: "-1",
        children: ["first_node", "second_node"],
        changes: [
            {
                id: "first_node",
                change: 1
            },
            {
                id: "second_node",
                change: 1
            },
            {
                id: "third_node",
                change: -1
            }],
        moveNull: true
    },
    "first_node": {
        from: [0,0],
        to: [5,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    },
    "second_node": {
        from: [6,0],
        to: [10,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    },
    "third_node": {
        from: [3,0],
        to: [7,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    }
}
//console.log(JSON.stringify(PathManager.unfoldCompact(PathManager.getChanges())));
console.log(PathManager.convertCompact());



console.log("++++++++++++++++++++++++++++++++++++++++++10 test -> completeChanges");
PathManager.path_tree = {
    "top_node": {
        from: [0,0],
        to: [0,0],
        playing: false,
        parent: "-1",
        children: ["first_node", "second_node", "third_node"],
        changes: [],
        moveNull: true
    },
    "first_node": {
        from: [0,0],
        to: [5,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    },
    "second_node": {
        from: [6,0],
        to: [10,0],
        playing: false,
        parent: "top_node",
        children: [],
        changes: [],
        moveNull: false
    },
    "third_node": {
        from: [3,0],
        to: [7,0],
        playing: false,
        parent: "top_node",
        children: ["layer_2_node_1", "layer_2_node_2", "layer_2_node_3"],
        changes: [],
        moveNull: false
    },
    "layer_2_node_1": {
        from: [0,0],
        to: [1,0],
        playing: false,
        parent: "third_node",
        children: [],
        changes: [],
        moveNull: true
    },
    "layer_2_node_2": {
        from: [0,0],
        to: [1,0],
        playing: false,
        parent: "third_node",
        children: [],
        changes: [],
        moveNull: true
    },
    "layer_2_node_3": {
        from: [0,0],
        to: [1,0],
        playing: false,
        parent: "third_node",
        children: ["layer_3_node_1"],
        changes: [],
        moveNull: true
    },
    "layer_3_node_1": {
        from: [0,0],
        to: [0,1],
        playing: false,
        parent: "layer_2_node_3",
        children: [],
        changes: [],
        moveNull: false
    }
}

console.log(PathManager.convertCompact());

//console.log(JSON.stringify(PathManager.unfoldCompact(PathManager.getChanges())));


//save("log-" + Date.now() + ".json")