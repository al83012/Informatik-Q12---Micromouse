import { Actions, Action } from "./Actions.js";
//import { Utils } from "./Utils.ts";

export class BackendManager {
    in_button_active = [true, false, false];
    in_selected_squares = []; // e.g. [[2,4],[3,1]]
    in_maze = {
        "visited": [],
        "discovered": [[0, 0]],
        "walls": [], // e.g. [[0,0, 0,1], [1,0, 1,1]] //wall between 00 and 01 as well as 10 and 11
        "goals": [],
        "paths": []
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
    algorithms = ["A*", "Dijkstra", "A* + Dijkstra"];
    in_algorithm = "A*";
    backend = null;

    f_sync = [];
    b_sync = [];


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
                break;
        }
    }

    f_handleError(body) {
        console.log("--------------------------------------------------------");
        console.log("[F] An Error occurred on the Frontend:");
        console.log("[F->Error]: " + body.error);
        console.log("--------------------------------------------------------");
    }

    b_handleUpdate(data) {
        console.log(data);
        switch (data.type) {
            case "":
                break;

            default:
                this.b_sync.push(Actions.b_error("recv", "incorrect_data", ["type"]));
                break;
        }
    } //backend

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

        return actions;
    }
}