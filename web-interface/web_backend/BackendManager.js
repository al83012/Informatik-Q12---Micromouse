import { Actions, Action } from "./Actions.js";
//import { Utils } from "./Utils.ts";

export class BackendManager {
    in_button_active = [true, false, false];
    in_selected_squares = []; // e.g. [[2,4],[3,1]]
    in_maze = {
        "visited": [],
        "discovered": [[0, 0], [0, 1], [0, 2], [1,0], [2,0], [3,0]],
        "walls": [], // e.g. [[0,0, 0,1], [1,0, 1,1]] //wall between 00 and 01 as well as 10 and 11
        "goals": [],
        "paths": [] //TODO: Think of overlapping paths
    };
    in_mouse = {
        "pos": [0, 0],
        "rotation": 0, // 0 up clockwise
        "direction": "n", // rotation in n-s-w-e //maybe unused
        "sensors": {
            "left_1": 0,
            "left_2": 0,

            "front_1": 0,
            "front_2": 0,

            "right_1": 0,
            "right_2": 0,
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
        }
    }

    f_handleError(body) {
        console.log("\x1b[31m--------------------------------------------------------");
        console.log("[F] An Error occurred on the Frontend:");
        console.log("[F->Error]: " + body.error);
        console.log("--------------------------------------------------------\x1b[33m");
    }

    b_handlePost(message) {
        /*console.log("--------------------------------------------------------------------");
        console.log(data, data[0]["MicromouseEvent"]);
        console.log("--------------------------------------------------------------------");*/
        for (let data in message) {
            if (data["MicromouseEvent"] !== undefined) {
                let event = data["MicromouseEvent"];
                if (event["UpdatePosition"] !== undefined) {
                    this.b_updateMousePosition(event["UpdatePosition"]);
                    return;
                } else if (event["UpdateMap"] !== undefined) {
                    this.b_updateMap(event["UpdateMap"]);
                    return;
                }
            } else {
                this.b_sync(Actions.b_error("recv", "incorrect_data", ["type"]));
            }
        }
    } //backend

    b_updateMap(data) {
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
            for (let i = 0; i < wall_disc.length; i++) {
                let disc = wall_disc[i];
                if (disc["new_status"] === "Visited") {
                    this.b_add_wall(disc["from_cell"]["x"], disc["from_cell"]["y"], disc["in_direction"]);
                } else if (disc["new_status"]["Exists"] !== undefined) {
                    if (disc["new_status"]["Exists"]) {
                        if (!this.b_wall_exists(disc["from_cell"]["x"], disc["from_cell"]["y"], disc["in_direction"])) {
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
                this.f_sync.push(Actions.discover_wall);
                break;
            case "NegX":
                this.in_maze.walls.push([x, y, x-1, y]);
                break;
            case "PosY":
                this.in_maze.walls.push([x, y, x, y+1]);
                break;
            case "NegY":
                this.in_maze.walls.push([x, y, x, y-1]);
                break;
        }
    }

    b_wall_exists(x, y, dir) {

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