import { Actions, Action } from "./Actions.js";

export class BackendManager {
    in_button_active = [false, false, false];
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
        "sensors": {}
    };
    in_console = ["[D] TestDebug", "[D] TestDebug 2", "[D] TestDebug 3", "[D] TestDebug 4", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
    client;

    constructor() {}

    set_backend(client) {
        this.client = client;
    }

    f_handleUpdate(client, data) {}

    b_handleUpdate() {}

    get_full() {
        let actions = [];
        actions.push(Actions.change_button(0, this.in_button_active[0]));
        actions.push(Actions.change_button(1, this.in_button_active[1]));
        actions.push(Actions.change_button(2, this.in_button_active[2]));

        for (let message of this.in_console) {
            actions.push(Actions.add_message(message));
        }

        return actions;
    }
}