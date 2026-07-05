const { PathManager } = require("./PathManager.ts");

PathManager.top_node_id = "top_node";

console.log("++++++++++++++++++++++++++++++++++++++++++");
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

console.log("++++++++++++++++++++++++++++++++++++++++++");
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

console.log("++++++++++++++++++++++++++++++++++++++++++");
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

console.log("++++++++++++++++++++++++++++++++++++++++++");
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

console.log("++++++++++++++++++++++++++++++++++++++++++");
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





console.log(JSON.stringify(PathManager.unfoldCompact(PathManager.getChanges())));

console.log("++++++++++++++++++++++++++++++++++++++++++");
