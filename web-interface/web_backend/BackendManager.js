import { Actions, Action } from "./Actions.js";
//import { Utils } from "./Utils.ts";
import { PathManager } from "./PathManager.ts";

export class BackendManager {
    in_button_active = [true, false, false];
    in_selected_squares = []; // e.g. [[2,4],[3,1]]
    in_maze = {
        visited: [],
        discovered: [[0,0]],//[[0, 0], [0, 1], [0, 2], [1,0], [2,0], [3,0]],
        walls: [], // e.g. [[0,0, 0,1], [1,0, 1,1]] //wall between 00 and 01 as well as 10 and 11
        goals: [],
        top_path_node: -1,
        path_tree: {
            /*example
            0: {
                from: [0,0],
                to: [0,0],
                children: [],
                playing: false,
                changes: [
                    {id: 1, change: 1} //changes: 1=add ; -1=remove (no longer in children) ; 0=children have changes
                ],
            }*/
        },
        /*paths: {
            /*example:
            * <id>: {
            *   children: [],
            *   current_iteration: 0,
            *   iterations: [
            *       {
            *           playing: false,
            *           segments: {},
            *           changes: {}
            *       }
            *   ]
            * }
            * I want to store the last 2-3 iterations + the iteration the Animation is currently on*/
            /*0: {
                children: [],
                current_iteration: 0,
                currently_playing: -1, //iteration
                last_played: -1,
                iterations: {
                    0: {
                        playing: false,
                        segments: {},
                        changes: {}
                    }
                }
            }
        }*/
    };
    in_mouse = {
        "pos": [0, 0],
        "rotation": 0, // 0 up clockwise
        "direction": "n", // rotation in n-s-w-e //maybe unused
        "sensors": {
            "battery": 99,
            /*
            "left_1": 0,
            "left_2": 0,

            "front_1": 0,
            "front_2": 0,

            "right_1": 0,
            "right_2": 0,*/
        }
    };
    in_console = ["[B] Connected"];
    algorithms = [
        {name: "A*", id: "AStar", config: {}},
        {name: "Dijkstra", id: "Dijkstra", config: {}},
        {name: "A* + Dijkstra", id: "AStarDijkstra", config: {}},
        {name: "Depth First Search", id: "DepthFirst", config: {"forward_first": true}}
    ];

    in_algorithm = "A*";
    in_is_loading = false;
    backend = null;

    f_sync = [];
    b_sync(action) {
        if (this.backend === null) return;
        this.backend.sendUTF(JSON.stringify(action));
    }


    constructor() {}

    set_backend(client) {
        this.backend = client;
    }

    f_handleUpdate(res) {
        // send the newest path updates
        /*if (this.in_maze.top_path_node !== -1) {
            console.log("Sending path updates: " + this.in_maze.path_tree[this.in_maze.top_path_node].changes.length);
            if (this.in_maze.path_tree[this.in_maze.top_path_node].changes.length !== 0) {
                let changes = this.calculate_path_change(this.in_maze.top_path_node, false);
                //console.log(changes);
                let updatePathAction = Actions.update_path(changes, this.in_maze.path_tree);
                //console.log(updatePathAction);
                this.f_sync.push(updatePathAction);
            }
        }*/
        if (PathManager.hasTopNode()) {
            if (PathManager.canPlay() && false) {
                console.log("Sending path updates");
                let changes = PathManager.unfold(PathManager.getChanges());
                this.f_sync.push(new Action({action: "update_path", data: {path: {data: changes}, id: changes[0][changes[0].length-1]}}));
            }
        }

        res.send(Actions.toString(this.f_sync));
        this.f_sync = [];
    } //frontend

    f_handlePost(data) {
        switch (data.action) {
            case "button_clicked":
                if (data.button_id === 0 && this.in_button_active[0]) {
                    this.in_button_active[0] = false;
                    this.in_button_active[1] = true;
                    this.in_button_active[2] = true;
                    this.f_sync.push(Actions.update_button(0, false));
                    this.f_sync.push(Actions.update_button(1, true));
                    this.f_sync.push(Actions.update_button(2, true));
                }
                break;
            case "maze_clicked":
                if (this.in_selected_squares === [[data.x, data.y]]) {
                    this.in_button_active[0] = false;
                    this.f_sync.push(Actions.update_button(0, false));
                    //TODO: implement the selection and deselection
                } else {
                    this.in_button_active[0] = true;
                    this.f_sync.push(Actions.update_button(0, true));
                }
                break;
            case "algorithm_selected":
                //TODO: send the algorithm to the backend
                this.in_algorithm = data.algorithm;
                this.f_sync.push(Actions.update_algorithm(data.algorithm));
                for (var algo in this.algorithms) {
                    if (algo.name === data.algorithm) {
                        this.f_sync.push(Actions.show_loading()); //TODO: add a "wait for complete" to hide the loading screen
                        this.b_sync(Actions.b_strategy_change({is: false}, false,
                            {is: true, config: {name: algo.id, config: algo.config}}));
                        break;
                    }
                }
                break;
            case "path_update":
                // lock the path from updating and send only the
                // other path updates or unlock the path depending on sent state
                /*let id = data.id;
                let it = data.it;
                let state = data.state;
                if (state === "started") {
                    this.in_maze.paths[id].currently_playing = it;
                    this.in_maze.paths[id].last_played = it;
                    this.in_maze.paths[id].iterations[it].playing = true;
                } else if (state === "finished") {
                    this.in_maze.paths[id].currently_playing = -1;
                    this.in_maze.paths[id].iterations[it].playing = false;
                }*/
                //this.in_maze.path_tree[data.id].playing = data.state === "started";
                PathManager.setPlayingState(data.state);
                break;
            case "double_path_update":
                //TODO: read id and iteration from packet and calculate
                // new displayed path and send or go to error
                console.log("Not implemented -> Collision with path updates");
                break;
        }
    }

    f_handleError(body) {
        console.log("\x1b[31m--------------------------------------------------------");
        console.log("[F] An Error occurred on the Frontend:");
        console.log("[F->Error]: " + body.error);
        console.log("--------------------------------------------------------\x1b[33m");
    }

    generateHash(string) {
        let hash = 0;
        for (const char of string) {
            hash = (hash << 5) - hash + char.charCodeAt(0);
            hash |= 0; // Constrain to 32bit integer
        }
        return hash;
    };

    b_handlePost(message) {
        /*console.log("--------------------------------------------------------------------");
        console.log(data, data[0]["MicromouseEvent"]);
        console.log("--------------------------------------------------------------------");*/

        console.log("..........................................................");
        console.log(JSON.stringify(message));
        console.log(";;;;;;;;;;;;;");

        main: for (let index in message) {
            let data = message[index];

            if (data["MicromouseEvent"] !== undefined) {
                let event = data["MicromouseEvent"];
                if (event["UpdatePosition"] !== undefined) {
                    this.b_updateMousePosition(event["UpdatePosition"]);
                    continue main;
                } else if (event["UpdatedMap"] !== undefined) {
                    this.b_updatedMap(event["UpdatedMap"]);
                    continue main;
                }
            } else if (data["VisualEvent"] !== undefined) {
                let event = data["VisualEvent"];
                if (event["PathVisualEvent"] !== undefined) {
                    //let hash = this.generateHash(JSON.stringify(event["PathVisualEvent"]["associated_node"]));
                    let id = JSON.stringify(event["PathVisualEvent"]["associated_node"]);

                    if (typeof event["PathVisualEvent"]["ty"] === "string") {
                        if (event["PathVisualEvent"]["ty"] === "Prune") {
                            PathManager.pruneNode(id);
                            /*this.delete_children(hash);
                            let parent = this.in_maze.path_tree[this.in_maze.path_tree[hash].parent];
                            try {
                                parent.changes.push({id: hash, change: -1});
                                this.update_parent_changes(this.in_maze.path_tree[hash].parent);
                            } catch (TypeError) {
                                console.log("ERROR");
                                console.log(hash)
                                console.log(TypeError);
                                console.log(this.in_maze.path_tree);
                            }
                            this.correct_paths();*/
                            continue main;
                        } else if (event["PathVisualEvent"]["ty"] === "Remove") {
                            //Maybe ignore
                            //this.correct_paths();
                            PathManager.removeNode(id);
                            continue main;
                        }
                    } else if (typeof event["PathVisualEvent"]["ty"] === "object") {
                        /*if (this.in_maze.top_path_node === -1) {
                            this.in_maze.path_tree[hash] = {
                                from: [0, 0],
                                to: [0, 0],
                                children: [],
                                changes: [],
                                parent: -1,
                                playing: false,
                                is_rotate: true,
                            }
                            this.in_maze.top_path_node = hash;
                        }*/

                        let visual_event = event["PathVisualEvent"];
                        if (visual_event["ty"]["Create"] !== undefined) {
                            let create = visual_event["ty"]["Create"];
                            let from = [create["path"]["from"]["pos"]["x"], create["path"]["from"]["pos"]["y"]];
                            let to = [create["path"]["to"]["pos"]["x"], create["path"]["to"]["pos"]["y"]];

                            //let new_node_hash = this.generateHash(JSON.stringify(create["leads_to_child_node"]));
                            let newId = JSON.stringify(create["leads_to_child_node"]);
                            PathManager.addNode(from, to, newId, id);

                            /*this.in_maze.path_tree[new_node_hash] = {
                                from: from,
                                to: to,
                                children: [],
                                changes: [],
                                parent: hash,
                                playing: false,
                                is_rotate: from[0] === to[0] && from[1] === to[1],
                            };

                            this.in_maze.path_tree[hash].children.push(new_node_hash);
                            this.in_maze.path_tree[hash].changes.push({id: new_node_hash, change: 1});
                            this.correct_paths();
                            this.update_parent_changes(hash);*/
                            continue main;
                        }
                    }
                }
            } else if (data["Debug"] !== undefined) {
                this.f_sync.push(Actions.add_message(data["Debug"]));
            } else {
                //this.b_sync(Actions.b_error("recv", "incorrect_data", ["type"]));
            }
        }
        console.log("..........................................................--");
    } //backend

    update_parent_changes(hash) {
        let node = this.in_maze.path_tree[hash];
        if (node === undefined) {return}
        if (node.parent === -1) {return}
        let parent = this.in_maze.path_tree[node.parent];
        if (parent.changes.some(change => change.id === hash)) {return}
        parent.changes.push({id: hash, change: 0});
        this.update_parent_changes(parent.parent);
    }

    correct_paths() {
        let top_path_node = this.in_maze.top_path_node;
        if (this.in_maze.path_tree[top_path_node].changes.length === 0) {return}
        this.correct_sub_paths(top_path_node);
    }

    correct_sub_paths(hash) {
        let node = this.in_maze.path_tree[hash];
        node.changes = node.changes.filter(change => {
            return node.changes.some(change2 => {
                return !(change.id === change2.id && change.change === -(change2.change));
            });
        });
        for (let child of node.children) {
            this.correct_sub_paths(child);
        }
    }

    delete_children(hash) {
        let node = this.in_maze.path_tree[hash];
        for (let child of node.children) {
            this.delete_children(child);
            node.changes.push({id: child, change: -1});
        }
        this.in_maze.path_tree[node.parent].children =
            this.in_maze.path_tree[node.parent].children.filter(child => child !== hash);
        if (this.in_maze.path_tree[node.parent].children === undefined) {
            this.in_maze.path_tree[node.parent] = [];
        }
    }

    b_updatedMap(data) {
        let cell_disc = data["cell_discoveries"];
        let wall_disc = data["wall_discoveries"];

        if (cell_disc.length !== 0) {
            for (let i = 0; i < cell_disc.length; i++) {
                let disc = cell_disc[i];
                switch (disc["new_status"]) {
                    case "Discovered":
                        this.f_sync.push(Actions.discover_tile(disc["at_cell"]["x"], disc["at_cell"]["y"], this.in_maze.discovered, false));
                        this.in_maze.discovered.push([disc["at_cell"]["x"], disc["at_cell"]["y"]]);
                        break;
                    case "Visited":
                        this.in_maze.visited.push([disc["at_cell"]["x"], disc["at_cell"]["y"]]);
                        break;
                    default:
                        break;
                }
            }
        }

        if (wall_disc.length !== 0) {
            console.log("++++++++++++++++++++++++++++++++++++++++++++++++++++");
            console.log(JSON.stringify(wall_disc));
            console.log("----------------------------------------------------");
            for (let i = 0; i < wall_disc.length; i++) {
                let disc = wall_disc[i];
                console.log(JSON.stringify(disc));
                if (disc["new_status"] === "Visited") {
                    if (!this.b_wall_exists(disc["from_cell"]["x"], disc["from_cell"]["y"], disc["in_direction"])) {
                        this.b_add_wall(disc["from_cell"]["x"], disc["from_cell"]["y"], disc["in_direction"]);
                    }
                } else if (disc["new_status"]["Exists"] !== undefined) {
                    if (disc["new_status"]["Exists"]) {
                        console.log("++Exists: " + disc["new_status"]["Exists"]);
                        if (!this.b_wall_exists(disc["from_cell"]["x"], disc["from_cell"]["y"], disc["in_direction"])) {
                            console.log("++++Added Wall");
                            this.b_add_wall(disc["from_cell"]["x"], disc["from_cell"]["y"], disc["in_direction"]);
                        }
                    }
                }

            }
        }
    }

    b_updateMousePosition(data) {
        let x_old = this.in_mouse.pos[0];
        let y_old = this.in_mouse.pos[1];
        this.in_mouse.pos[0] = data["pos"]["x"];
        this.in_mouse.pos[1] = data["pos"]["y"];
        let rot_old = this.in_mouse.rotation;
        switch (data["dir"]) {
            case "PosX":
                this.in_mouse.direction = "e";
                this.in_mouse.rotation = 1;
                break;
            case "NegX":
                this.in_mouse.direction = "w";
                this.in_mouse.rotation = 3;
                break;
            case "PosY":
                this.in_mouse.direction = "s";
                this.in_mouse.rotation = 2;
                break;
            case "NegY":
                this.in_mouse.direction = "n";
                this.in_mouse.rotation = 0;
                break;
        }
        if (x_old !== this.in_mouse.pos[0] || y_old !== this.in_mouse.pos[1]) {
            this.f_sync.push(Actions.move_mouse(
                x_old,
                y_old,
                this.in_mouse.pos[0],
                this.in_mouse.pos[1]
            ));
        }
        if (rot_old !== this.in_mouse.rotation) {
            this.f_sync.push(Actions.rotate_mouse(
                x_old,
                this.in_mouse.rotation
            ));
        }
    }

    b_add_wall(x, y, dir) {
        switch (dir) {
            case "PosX":
                this.in_maze.walls.push([x, y, x+1, y]);
                this.f_sync.push(Actions.discover_wall(x, y, x+1, y));
                break;
            case "NegX":
                this.in_maze.walls.push([x-1, y, x, y]);
                this.f_sync.push(Actions.discover_wall(x-1, y, x, y));
                break;
            case "PosY":
                this.in_maze.walls.push([x, y, x, y+1]);
                this.f_sync.push(Actions.discover_wall(x, y, x, y+1));
                break;
            case "NegY":
                this.in_maze.walls.push([x, y-1, x, y]);
                this.f_sync.push(Actions.discover_wall(x, y-1, x, y));
                break;
        }
    }

    b_wall_exists(x, y, dir) { //walls are always constructed from left to right or top to bottom
        switch (dir) {
            case "PosX":
                return this.in_maze.walls.some(wall => wall[0] === x && wall[1] === y && wall[2] === x+1 && wall[3] === y);
            case "NegX":
                return this.in_maze.walls.some(wall => wall[0] === x-1 && wall[1] === y && wall[2] === x && wall[3] === y);
            case "PosY":
                return this.in_maze.walls.some(wall => wall[0] === x && wall[1] === y && wall[2] === x && wall[3] === y+1);
            case "NegY":
                return this.in_maze.walls.some(wall => wall[0] === x && wall[1] === y-1 && wall[2] === x && wall[3] === y);
        }
    }

    calculate_path_change(id, ignore_changes) {

        let nodes = [id];
        if (this.in_maze.top_path_node === -1) {
            throw new Error("No path found");
        }
        if (this.in_maze.path_tree[nodes[0]].changes.length === 0 && !ignore_changes) {
            return [[id, 0, 1]];
        }

        let next_nodes = [];
        let changes = [];

        let is_first = true;

        if (ignore_changes) {
            changes.push([id, 0, 1]);
        }

        while (true) {
            label_nodes: for (let node of nodes) {
                let is_rotate = false;
                /*if (this.in_maze.path_tree[node].parent === -1) {
                    nodes = this.in_maze.path_tree[node].children.slice();
                    continue label_nodes;
                }*/

                if (!ignore_changes) {
                    label_changes: for (let change of this.in_maze.path_tree[node].changes) {
                        next_nodes.push(change.id);
                        if (this.in_maze.path_tree[node].playing) {
                            if (is_first) {
                                changes.push([change.id, (this.in_maze.path_tree[change.id].is_rotate ? 9 : 3), 1]);
                                is_first = false;
                                continue label_changes;
                            }
                            changes.push([change.id, (this.in_maze.path_tree[change.id].is_rotate ? 9 : 3), 0]);
                            continue label_changes;
                        }
                        if (is_first) {
                            changes.push([change.id, (this.in_maze.path_tree[change.id].is_rotate ? 9 : change.change), 1]);
                            is_first = false;
                            continue label_changes;
                        }
                        changes.push([change.id, (this.in_maze.path_tree[change.id].is_rotate ? 9 : change.change), 0]);
                    }
                    this.in_maze.path_tree[node].changes = []; //clear changes
                } else {
                    for (let child of this.in_maze.path_tree[node].children) {
                        next_nodes.push(child);
                        if (is_first) {
                            changes.push([child, (this.in_maze.path_tree[child].is_rotate ? 9 : 0), 1]);
                            is_first = false;
                            continue;
                        }
                        changes.push([child, (this.in_maze.path_tree[child].is_rotate ? 9 : 0), 0]);
                    }
                }
            }
            is_first = true;
            //changes[0][2] = 1; //set end of this layer at position 0 -> anim is calculated backwards
            if (next_nodes.length === 0) {
                break;
            }
            nodes = next_nodes.slice();
            next_nodes = [];
        }

        changes = changes.filter(change => change[1] !== 9); //filter rotate nodes

        //delete overlapping
        for (let i = 0; i < changes.length; i++) {
            let node = this.in_maze.path_tree[changes[i][0]];

            for (let j = i+1; j < changes.length; j++) {
                let node2 = this.in_maze.path_tree[changes[j][0]];

                if (node.from[0] === node2.from[0] && node.from[1] === node2.from[1] &&
                node.to[0] === node2.to[0] && node.to[1] === node2.to[1]) {
                    if (changes[i][1] === 3 || changes[j][1] === 3) {
                        changes[i][1] = 3;
                        changes[j][1] = 3;
                        if (changes[i][2] === 1) {
                            changes[i-1][2] = 1;
                        }
                        if (changes[j][2] === 1) {
                            changes[j-1][2] = 1;
                        }
                        continue;
                    }
                    changes[i][1] = 0; //keep them the same
                    changes[j][1] = 0;
                }
            }
        }

        return changes.filter(change => change[1] !== 3); //filter playing nodes




        //assuming that path_changes follows this format and that every part (same/keep,remove,add) is in changes:
        //[<changes>]
        //<change> = [<coord x start>, <coord y start>, <coord x end>, <coord y end>, <change id>]

        //Deprecated
        /*if (this.in_maze.paths[path_id] === undefined) {
            //create new path
            this.in_maze.paths[path_id] = [];
        }

        let path_coords = [];
        let path_change_syncs = [];
        let new_path = [];

        for (let change of path_changes) {
            let part_coords = [];
            for (let x_dif = 0; x_dif < change[2] - change[0]; x_dif++) {
                path_coords.push([change[0] + x_dif, change[1]]);
            }

            for (let x_dif = 0; x_dif > change[2] - change[0]; x_dif--) {
                path_coords.push([change[0] + x_dif, change[1]]);
            }

            for (let y_dif = 0; y_dif < change[3] - change[1]; y_dif++) {
                path_coords.push([change[0], change[1] + y_dif]);
            }

            for (let y_dif = 0; y_dif > change[3] - change[1]; y_dif--) {
                path_coords.push([change[0], change[1] + y_dif]);
            }

            part_coords.push(change[4]);

            path_coords.push(part_coords);
        }

        for (const [key, value] of this.in_maze.paths) {
            if (key === path_id) continue;

            for (let part_coord of path_coords) {
                for (let maze_coord of value) {
                    if (Math.min(part_coord.length, maze_coord.length) <= 2) continue; //skip parts that are to short to be tested //TODO: think of a way to test those edge cases
                    let is_same = false;
                    for (let i = 0; i < Math.min(part_coord.length, maze_coord.length); i++) {
                        if (part_coord[i] === maze_coord[i] && part_coord[i+1] === maze_coord[i+1]
                            && part_coord[i+2] === maze_coord[i+2] && part_coord[i+3] === maze_coord[i+3]) {
                            is_same = true;
                        }
                    }
                    if (!is_same) {
                        path_change_syncs.push(JSON.parse(JSON.stringify(part_coord)));
                    }
                }

                let id = part_coord.pop(); //remove change id
                if (id !== -1) {
                    new_path.push(part_coord);
                }
            }
        }

        this.in_maze.paths[path_id] = new_path;*/
    }

    get_full() {
        let actions = [];
        actions.push(Actions.update_button(0, this.in_button_active[0]));
        actions.push(Actions.update_button(1, this.in_button_active[1]));
        actions.push(Actions.update_button(2, this.in_button_active[2]));
        actions.push(Actions.update_con_status(this.backend !== null));

        for (let message of this.in_console) {
            actions.push(Actions.add_message(message));
        }

        for (let algo of this.algorithms) {
            actions.push(Actions.add_algorithm(algo));
        }

        for (const [sensor, value] of Object.entries(this.in_mouse.sensors)) {
            actions.push(Actions.update_sensor(sensor, value));
        }

        for (const wall of this.in_maze.walls) {
            actions.push(Actions.discover_wall(wall[0], wall[1], wall[2], wall[3]));
        }

        /*if (this.in_maze.top_path_node !== -1) {
            let path = this.calculate_path_change(this.in_maze.top_path_node, true);
            actions.push(Actions.complete_path(path, this.in_maze.path_tree));
        }*/

        for (let i = 0; i < this.in_maze.discovered.length; i++) {
            let x = this.in_maze.discovered[i][0];
            let y = this.in_maze.discovered[i][1];
            actions.push(Actions.discover_tile(x, y, this.in_maze.discovered, true));
        }

        actions.push(Actions.move_mouse(0, 0, this.in_mouse.pos[0], this.in_mouse.pos[1]));
        actions.push(Actions.rotate_mouse(0, this.in_mouse.rotation));

        if (this.in_is_loading) {
            actions.push(Actions.show_loading());
        }

        return actions;
    }
}