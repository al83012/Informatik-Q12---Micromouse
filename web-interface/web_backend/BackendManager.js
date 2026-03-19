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

    constructor() {}

    f_handleUpdate() {}

    b_handleUpdate() {}
}