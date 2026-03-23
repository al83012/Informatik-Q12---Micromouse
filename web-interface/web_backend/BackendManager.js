import { Actions, Action } from "./Actions.js";

export class BackendManager {
    in_button_active = [true, false, false];
    in_selected_squares = []; // e.g. [[2,4],[3,1]]
    in_maze = {
        "visited": [],
        "walls": [], // e.g. [[0,0, 0,1], [1,0, 1,1]] //wall between 00 and 01 as well as 10 and 11
        "goals": [],
        "path": []
    };
    in_mouse = {
        "pos": [0, 0],
        "rotation": 0, // 0 up clockwise
        "sensors": {
            "left_1": 0,
            "left_2": 0,

            "front_1": 0,
            "front_2": 0,

            "right_1": 0,
            "right_2": 0,
        }
    };
    in_console = ["[D] TestDebug", "[D] TestDebug 2", "[D] TestDebug 3", "[D] TestDebug 4", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
    backend;

    sync = [];

    constructor() {}

    set_backend(client) {
        this.backend = client;
    }

    f_handleUpdate(res) {
        res.send(Actions.toString(this.sync));
        this.sync = [];
    } //frontend

    f_handlePost(data) {
        switch (data.action) {
            case "button_clicked":
                if (data.button_id === 0 && this.in_button_active[0]) {
                    this.in_button_active[0] = false;
                    this.in_button_active[1] = true;
                    this.in_button_active[2] = true;
                    this.sync.push(Actions.update_button(0, false));
                    this.sync.push(Actions.update_button(1, true));
                    this.sync.push(Actions.update_button(2, true));
                }
                break;
            case "maze_clicked":
                if (this.in_selected_squares === [[data.x, data.y]]) {
                    this.in_button_active[0] = false;
                    this.sync.push(Actions.update_button(0, false));
                    //TODO: implement the selection and deselection
                } else {
                    this.in_button_active[0] = true;
                    this.sync.push(Actions.update_button(0, true));
                }
                break;
        }
    }

    b_handleUpdate(data) {} //backend

    get_full() {
        let actions = [];
        actions.push(Actions.update_button(0, this.in_button_active[0]));
        actions.push(Actions.update_button(1, this.in_button_active[1]));
        actions.push(Actions.update_button(2, this.in_button_active[2]));

        for (let message of this.in_console) {
            actions.push(Actions.add_message(message));
        }

        return actions;
    }
}