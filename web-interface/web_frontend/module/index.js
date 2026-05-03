let is_square_selected = false;
let selected_square;
//first is current command like center/point/reset and second is the state of command like running/finished/stopped
let current_state = ["reset", "finished"];
let goal = [15, 15]; //the coords for going somewhere (cmd point)


import {
    AnimationHandler,
    AnimBorderInner,
    AnimFadeIn,
    AnimFadeOut,
    AnimBackgroundColor,
    AnimSlideColor,
    AnimGroup,
    AnimCssChange,
    AnimMoveMultiples,
    AnimRotate,
    generatePathAnimGroup
} from "./Animation.js"
import { StyleAdder } from "./style_adder.js";

export class Index {
    static animHandler;

    static squares = {};
    static algorithms = [];

    static eventLoad() {
        //creating error feedback to backend
        window.onerror = function (e) {
            let request = new XMLHttpRequest();
            request.open("POST", document.location.origin + "/error");
            request.setRequestHeader("Content-Type", "Application/json");
            request.send(JSON.stringify({
                error: e,
                stack: e.stack,
            }));
        }

        //move info-link_pill to respective position
        let info_link_pill = document.getElementsByClassName("link-info_pill")[0];
        info_link_pill.style.top = (document.getElementsByClassName("header_container")[0].clientHeight + 10) + "px";
        info_link_pill.onclick = function () {
            document.location.replace(document.location.origin + "/info/info.html");
            //document.location.reload();
        }


        let squares = document.getElementsByClassName("maze_tile");

        for (let i = 0; i < squares.length; i++) {
            let square = squares[i];
            square.val_index = i;
            Index.squares[convert_index_to_coords(i)] = square;
            square.val_selected = false;
            square.addEventListener("click",
                function (e) {

                    let select = Index.squares[convert_index_to_coords(e.currentTarget.val_index)];

                    if (select.val_selected) {
                        unselect_square(select);
                        select.val_selected = false;
                        is_square_selected = false;
                    } else {
                        if (is_square_selected) {
                            unselect_square(selected_square);
                            selected_square.val_selected = false;
                        }
                        select_square(select);
                        select.val_selected = true;
                        is_square_selected = true;
                        selected_square = select;
                    }
                });
        }

        StyleAdder.disableForClass();

        //creating the AnimationHandler
        Index.animHandler = new AnimationHandler();
        window.setInterval(() => {
            let request = new XMLHttpRequest();
            request.addEventListener("load", function () {Index.handleUpdate(JSON.parse(this.responseText));});
            request.open("GET", document.location.origin + "/update", true);
            request.send();
        }, 100);
        window.setInterval(() => {Index.animHandler.nextFrame();}, 10)
        
        //window.setTimeout(() => init_maze(), 1000); //the timeout is only for the test animation which had a bug
        init_maze();


        //let borderAnim = new AnimBorderColor(100, Index.squares[[5, 5]], "darkblue", "cyan");
        //Index.animHandler.add(borderAnim);

        let request = new XMLHttpRequest();
        request.addEventListener("load", function () {console.log(this.responseText);Index.handleUpdate(JSON.parse(this.responseText));});
        request.open("GET", document.location.origin + "/update_full");
        request.send();
    }

    static handleUpdate(response) {
        response["actions"].forEach(action => {
            let data = action["data"];
            console.log(action.action);
            switch (action["action"]) {
                case "update_button":
                    updateControls(data["button_id"], data["state"]);
                    break;
                case "add_message":
                    add_message(data["message"]);
                    break;
                case "update_sensor":
                    update_sensors(data["sensor"], data["values"]);
                    break;
                case "add_algorithm":
                    add_algorithm(data["algorithm"]);
                    document.getElementById("algo_current").innerHTML = data["algorithm"];
                    break;
                case "update_algorithm":
                    document.getElementById("algo_current").innerHTML = data["algorithm"];
                    break;
                case "update_con_status":
                    document.getElementById("pill_con_dot").style.borderColor = (data["status"] ? "green" : "red");
                    document.getElementById("pill_con_text").innerHTML = (data["status"] ? "online" : "offline");
                    break;
                case "reset_maze":
                    reset_maze(data["animation"] === "true");
                    break;
                case "update_path":
                    Index.animHandler.addImmediate(displayPathChange(data["path"]));
                    break;
                case "discover_tile":
                    if (data["others"] === true) {
                        discoverTile(data["x"], data["y"], data["directions"], data.other_tiles_x, data.other_tiles_y);
                    } else {
                        discoverTile(data["x"], data["y"], data["directions"]);
                    }
                    break;
                case "move_mouse":
                    let card_move = document.getElementById("sys-mouse_card");
                    Index.animHandler.addImmediate(new AnimMoveMultiples(20, card_move, 34, data["x"], data["x_new"], data["y"], data["y_new"]));
                    break;
                case "rotate_mouse":
                    let card_rotate = document.getElementById("sys-mouse_card");
                    Index.animHandler.addImmediate(new AnimRotate(7, card_rotate, data["dir"], data["dir_new"]));
                    break;
                case "show_loading":
                    Index.show_loading_animation();
                    break;
                case "hide_loading":
                    Index.hide_loading_animation();
                    break;
            }
        });
    }

    static buttonStartStop() {
        //play_reset_animation();

        let request = new XMLHttpRequest();
        request.open("POST", document.location.origin + "/action");
        request.addEventListener("load", function () {});
        request.setRequestHeader("Content-Type", "Application/json");
        request.send(JSON.stringify({
            action: "button_clicked",
            button_id: 0,
        }));

        /*let pathGroup = new AnimGroup(5);

        for (let i = 0; i < 255; i++) {
            pathGroup.add(new AnimBorderInner(45, Index.squares[convert_index_to_coords(i)], 2, 2, 1, [1, 30, 30]));
        }
        pathGroup.add(new AnimBorderInner(15, Index.squares[[0,0]], 2, 2, 1));
        pathGroup.add(new AnimBorderInner(15, Index.squares[[1,0]], 2, 2, 1));
        pathGroup.add(new AnimBorderInner(15, Index.squares[[2,0]], 2, 2, 1));
        pathGroup.add(new AnimBorderInner(15, Index.squares[[3,0]], 2, 2, 1));
        pathGroup.add(new AnimBorderInner(15, Index.squares[[4,0]], 2, 2, 1));
        pathGroup.add(new AnimBorderInner(15, Index.squares[[5,0]], 2, 2, 1));
        pathGroup.add(new AnimBorderInner(15, Index.squares[[6,0]], 2, 2, 1));
        pathGroup.add(new AnimBorderInner(15, Index.squares[[7,0]], 2, 2, 1));
        pathGroup.add(new AnimBorderInner(15, Index.squares[[7,1]], 2, 2, 1));
        pathGroup.add(new AnimBorderInner(15, Index.squares[[7,2]], 2, 2, 1));
        pathGroup.add(new AnimBorderInner(15, Index.squares[[7,3]], 2, 2, 1));
        pathGroup.add(new AnimBorderInner(15, Index.squares[[7,4]], 2, 2, 1));
        pathGroup.add(new AnimBorderInner(15, Index.squares[[7,5]], 2, 2, 1));
        pathGroup.add(new AnimBorderInner(15, Index.squares[[7,6]], 2, 2, 1));
        pathGroup.add(new AnimBorderInner(15, Index.squares[[7,7]], 2, 2, 1));

        Index.animHandler.add(pathGroup);*/

        /*
        let request = new XMLHttpRequest();
        request.addEventListener("load", function () {console.log(this.responseText);});
        request.open("GET", document.location.origin + "/update");
        request.send();*/
    }

    static openPopAlgo() {
        console.log("OpenPopAlgo");
        let obj = document.getElementsByClassName("pop_window_algorithm_group")[0];
        Index.animHandler.add(new AnimFadeIn(20, obj, (o) => {o.style.opacity = "0";o.style.display = "block";}));
        //document.getElementsByClassName("pop_window_algorithm_group")[0].style.animation = "fade-in 0.2s";
    }

    static closePopAlgo() {
        let obj = document.getElementsByClassName("pop_window_algorithm_group")[0];
        Index.animHandler.add(new AnimFadeOut(20, obj, (o) => {o.style.display = "none";}));
    }

    static show_loading_animation() {
        document.getElementById("loading-container").style.display = "block";

        let main_group = new AnimGroup(5);

        for (let i = 0; i < 24; i++) { //cycle through all 24 elements
            let part = document.getElementById("loading_animation_" + i);
            let group = new AnimGroup(-1);
            group.add(new AnimCssChange(10, part, ["undiscovered"], "discovered"));
            group.add(new AnimCssChange(65, part, ["repl"], "highlight"));
            group.add(new AnimCssChange(30, part, ["highlight"], "repl"));
            group.add(new AnimCssChange(15, part, ["discovered"], "undiscovered"));
            main_group.add(group);
        }

        Index.animHandler.addRepeating(main_group, "loading_animation");
    }

    static hide_loading_animation() {
        document.getElementById("loading-container").style.display = "none";
        Index.animHandler.removeRepeating("loading_animation");
    }
}

function discoverTile(x, y, directions, x_other, y_other) {
    let index = co_crds_i([x, y]);
    let group = new AnimGroup(0);
    group.add(new AnimCssChange(10, document.getElementById("sys-arm_node_" + index),
        ["undiscovered"], "discovered"));
    for (let i = 0; i < directions.length; i++) {
        group.add(new AnimCssChange(10, document.getElementById("sys-arm_arm-" + directions[i] + "_" + index),
            ["undiscovered"], "discovered"));
    }

    if (!(x_other === null || x_other === undefined)) {
        for (let i = 0; i < x_other.length; i++) {

            switch (x_other[i]-x) {
                case 1:
                    group.add(new AnimCssChange(10, document.getElementById("sys-arm_arm-e_" + co_crds_i([x+1, y])),
                        ["undiscovered"], "discovered"));
                break;

                case -1:
                    group.add(new AnimCssChange(10, document.getElementById("sys-arm_arm-w_" + co_crds_i([x-1, y])),
                        ["undiscovered"], "discovered"));
                break;

                case 0:
                    switch (y_other[i]-y) {
                        case 1:
                            group.add(new AnimCssChange(10, document.getElementById("sys-arm_arm-n_" + co_crds_i([x, y+1])),
                                ["undiscovered"], "discovered"));
                        break;

                        case -1:
                            group.add(new AnimCssChange(10, document.getElementById("sys-arm_arm-s_" + co_crds_i([x, y-1])),
                                ["undiscovered"], "discovered"));
                        break;
                    }
                break;
            }
        }
    }
    Index.animHandler.addImmediate(group);
}

function flip_nswe_directions(directions) {
    let new_directions = [];
    for (let i = 0; i < directions.length; i++) {
        switch (directions[i]) {
            case "n":
                new_directions.push("s");
                break;
            case "s":
                new_directions.push("n");
                break;
            case "e":
                new_directions.push("w");
                break;
            case "w":
                new_directions.push("e");
                break;
        }
    }
    return new_directions;
}

//new function for initializing using the arm system
function init_maze() {
    for (let i = 0; i < 16*16; i++) {
        let coords = convert_index_to_coords(i);
        let tile = Index.squares[coords];

        let arm_container = document.createElement("div");
        arm_container.className = "sys-arm_container";
        tile.appendChild(arm_container);

        let node = document.createElement("div");
        node.className = "sys-arm_node repl undiscovered";
        node.id = "sys-arm_node_" + i;
        arm_container.appendChild(node);

        if (!(coords[1] === 0)) {
            let arm_up = document.createElement("div");
            arm_up.className = "sys-arm_arm sys-arm_arm-n repl undiscovered";
            arm_up.id = "sys-arm_arm-n_" + i;
            arm_container.appendChild(arm_up);
        }

        if (!(coords[1] === 15)) {
            let arm_down = document.createElement("div");
            arm_down.className = "sys-arm_arm sys-arm_arm-s repl undiscovered";
            arm_down.id = "sys-arm_arm-s_" + i;
            arm_container.appendChild(arm_down);
        }

        if (!(coords[0] === 0)) {
            let arm_left = document.createElement("div");
            arm_left.className = "sys-arm_arm sys-arm_arm-w repl undiscovered";
            arm_left.id = "sys-arm_arm-w_" + i;
            arm_container.appendChild(arm_left);
        }

        if (!(coords[0] === 15)) {
            let arm_right = document.createElement("div");
            arm_right.className = "sys-arm_arm sys-arm_arm-e repl undiscovered";
            arm_right.id = "sys-arm_arm-e_" + i;
            arm_container.appendChild(arm_right);
        }
    }

    //let animation = generatePathAnimGroup([[0, 0, 1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 5, 1, 5, 2, 5, 3, 5, 4, 5, 5, 0]], tiles);
    /*let path = [
        [0, 0, 1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 5, 1, 5, 2, 5, 3, 5, 4, 5, 5, 0],
        [6, 5, 7, 5, 8, 5, 9, 5, 9, 6, 9, 7, 9, 8, 1],
        [5, 6, 5, 7, 5, 8, 5, 9, 6, 9, 7, 9, 8, 9, -1],
        [9, 9, 9, 10, 10, 10, 0]
    ];
    let animation = displayPathChange(path);
    let path_second = [
        [0, 0, 1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 5, 1, 5, 2, 5, 3, 5, 4, 5, 5, 0],
        [6, 5, 7, 5, 8, 5, 9, 5, 9, 6, 9, 7, 9, 8, 9, 9, 9, 10, 10, 10, -1],
        [5, 6, 5, 7, 5, 8, 5, 9, 5, 10, 5, 11, 6, 11, 7, 11, 8, 11, 9, 11, 10, 11, 1]
    ];
    let animation_second = displayPathChange(path_second);
    Index.animHandler.add(animation);
    Index.animHandler.add(animation_second);*/
}

function displayPathChange(changed_path) {
    let tiles = [];
    let group_points = [];
    for (let g = 0; g < changed_path.length; g++) {
        group_points.push([changed_path[g][0], changed_path[g][1], changed_path[g][changed_path[g].length-1]]);
        group_points.push([changed_path[g][changed_path[g].length-3], changed_path[g][changed_path[g].length-2],
            changed_path[g][changed_path[g].length-1]]);

        let group = changed_path[g];
        let prev = [group[0], group[1]];
        tiles[[prev[0], prev[1]]] = [document.getElementById("sys-arm_node_" + co_crds_i([prev[0], prev[1]]))];
        for (let i = 2; i < group.length -1; i+=2) { //-1 to skip the group type
            if (prev[0] < group[i]) {
                tiles[[prev[0], prev[1]]].push(document.getElementById("sys-arm_arm-e_" + co_crds_i([prev[0], prev[1]])));
                tiles[[group[i], group[i+1]]] = [
                    document.getElementById("sys-arm_arm-w_" + co_crds_i([group[i], group[i+1]])),
                    document.getElementById("sys-arm_node_" + co_crds_i([group[i], group[i+1]]))
                ];
            }
            else if (prev[0] > group[i]) {
                tiles[[prev[0], prev[1]]].push(document.getElementById("sys-arm_arm-w_" + co_crds_i([prev[0], prev[1]])));
                tiles[[group[i], group[i+1]]] = [
                    document.getElementById("sys-arm_arm-e_" + co_crds_i([group[i], group[i+1]])),
                    document.getElementById("sys-arm_node_" + co_crds_i([group[i], group[i+1]]))
                ];
            }
            else if (prev[1] < group[i+1]) {
                tiles[[prev[0], prev[1]]].push(document.getElementById("sys-arm_arm-s_" + co_crds_i([prev[0], prev[1]])));
                tiles[[group[i], group[i+1]]] = [
                    document.getElementById("sys-arm_arm-n_" + co_crds_i([group[i], group[i+1]])),
                    document.getElementById("sys-arm_node_" + co_crds_i([group[i], group[i+1]]))
                ];
            }
            else if (prev[1] > group[i+1]) {
                tiles[[prev[0], prev[1]]].push(document.getElementById("sys-arm_arm-n_" + co_crds_i([prev[0], prev[1]])));
                tiles[[group[i], group[i+1]]] = [
                    document.getElementById("sys-arm_arm-s_" + co_crds_i([group[i], group[i+1]])),
                    document.getElementById("sys-arm_node_" + co_crds_i([group[i], group[i+1]]))
                ];
            }
            prev[0] = group[i];
            prev[1] = group[i+1];
        }
    }

    for (let i = 0; i < group_points.length; i++) {
        for (let j = i+1; j < group_points.length; j++) {
            if (math_pos(group_points[i][0]-group_points[j][0]) <= 1 &&
                math_pos(group_points[i][1]-group_points[j][1]) <= 1 &&
                !(math_pos(group_points[i][0]-group_points[j][0]) === 1 &&
                    math_pos(group_points[i][1]-group_points[j][1]) === 1)) {

                let prev = [group_points[i][0], group_points[i][1]];

                if (prev[0] < group_points[j][0]) {
                    if (group_points[j][2] === 0) {
                        tiles[prev].push(document.getElementById("sys-arm_arm-e_" + co_crds_i([prev[0], prev[1]])));
                        tiles[prev].push(
                            document.getElementById("sys-arm_arm-w_" + co_crds_i([group_points[j][0], group_points[j][1]])));
                    } else {
                        tiles[[group_points[j][0], group_points[j][1]]]
                            .push(document.getElementById("sys-arm_arm-e_" + co_crds_i([prev[0], prev[1]])));
                        tiles[[group_points[j][0], group_points[j][1]]].push(
                            document.getElementById("sys-arm_arm-w_" + co_crds_i([group_points[j][0], group_points[j][1]])));
                    }
                }
                else if (prev[0] > group_points[j][0]) {
                    if (group_points[j][2] === 0) {
                        tiles[prev].push(document.getElementById("sys-arm_arm-w_" + co_crds_i([prev[0], prev[1]])));
                        tiles[prev].push(
                            document.getElementById("sys-arm_arm-e_" + co_crds_i([group_points[j][0], group_points[j][1]])));
                    } else {
                        tiles[[group_points[j][0], group_points[j][1]]]
                            .push(document.getElementById("sys-arm_arm-w_" + co_crds_i([prev[0], prev[1]])));
                        tiles[[group_points[j][0], group_points[j][1]]].push(
                            document.getElementById("sys-arm_arm-e_" + co_crds_i([group_points[j][0], group_points[j][1]])));
                    }
                }
                else if (prev[1] < group_points[j][1]) {
                    if (group_points[j][2] === 0) {
                        tiles[prev].push(document.getElementById("sys-arm_arm-s_" + co_crds_i([prev[0], prev[1]])));
                        tiles[prev].push(
                            document.getElementById("sys-arm_arm-n_" + co_crds_i([group_points[j][0], group_points[j][1]])));
                    } else {
                        tiles[[group_points[j][0], group_points[j][1]]]
                            .push(document.getElementById("sys-arm_arm-s_" + co_crds_i([prev[0], prev[1]])));
                        tiles[[group_points[j][0], group_points[j][1]]].push(
                            document.getElementById("sys-arm_arm-n_" + co_crds_i([group_points[j][0], group_points[j][1]])));
                    }
                }
                else if (prev[1] > group_points[j][1]) {
                    if (group_points[j][2] === 0) {
                        tiles[prev].push(document.getElementById("sys-arm_arm-n_" + co_crds_i([prev[0], prev[1]])));
                        tiles[prev].push(
                            document.getElementById("sys-arm_arm-s_" + co_crds_i([group_points[j][0], group_points[j][1]])));
                    } else {
                        tiles[[group_points[j][0], group_points[j][1]]]
                            .push(document.getElementById("sys-arm_arm-n_" + co_crds_i([prev[0], prev[1]])));
                        tiles[[group_points[j][0], group_points[j][1]]].push(
                            document.getElementById("sys-arm_arm-s_" + co_crds_i([group_points[j][0], group_points[j][1]])));
                    }
                }
            }
        }
    }

    return generatePathAnimGroup(changed_path, tiles);
}

//deprecated function
//moved function to backend and awaiting server protocol to rewrite the function
//according to data received from server
function dep_displayPathChange(path_old, path_new) {
    let groups = [];
    let c_group = [];
    let type = -2;
    for (let i = 0; i < path_old.length; i+=2) {
        if (path_old[i] === path_new[i] && path_old[i+1] === path_new[i+1]) {
            if (!(type === -2 || type === 0)) {
                c_group.push(type);
                groups.push(c_group);
                c_group = [];
            }
            c_group.push(path_new[i]);
            c_group.push(path_new[i+1]);
            type = 0;
        } else if (path_old[i] !== path_new[i] || path_old[i+1] !== path_new[i+1]) {
            if (!(type === -2 || type === -1 || type === 1)) {}
        }
    }
}

//deprecated function using the path system
//now replaced by the arm system
function dep_init_maze() {
    for (let i = 0; i < 16*16; i++) {
        let tile = Index.squares[convert_index_to_coords(i)];
        let container = document.createElement("div");
        container.className = "path_container";
        container.id = "path_container_" + i;

        let row_up = document.createElement("div");
        row_up.className = "path_row_tb"
        row_up.id = "path_row_up_" + i;

        let row_middle = document.createElement("div");
        row_middle.className = "path_row_middle";
        row_middle.id = "path_row_middle_" + i;

        let row_down = document.createElement("div");
        row_down.className = "path_row_tb";
        row_down.id = "path_row_down_" + i;

        let up = document.createElement("div");
        up.className = "path_tile_tb";
        up.id = "path_tile_up_" + i;

        let down = document.createElement("div");
        down.className = "path_tile_tb";
        down.id = "path_tile_down_" + i;

        let right = document.createElement("div");
        right.className = "path_tile_lr";
        right.id = "path_tile_right_" + i;

        let left = document.createElement("div");
        left.className = "path_tile_lr";
        left.id = "path_tile_left_" + i;

        let middle = document.createElement("div");
        middle.className = "path_tile_middle";
        middle.id = "path_tile_middle_" + i;

        row_up.appendChild(up);
        row_middle.appendChild(left);
        row_middle.appendChild(middle);
        row_middle.appendChild(right);
        row_down.appendChild(down);

        container.appendChild(row_up);
        container.appendChild(row_middle);
        container.appendChild(row_down);

        tile.appendChild(container);
    }

    //reset_maze(false);
}

function reset_maze(play_anim) {
    if (play_anim) {
        play_reset_animation((object) => {
            let children = object.children[0].children;
            for (let i = 0; i < children.length; i++) {
                children[i].className = children[i].className.replaceAll(" on", "");
            }
        });
    } else {
        for (let i = 0; i < 16*16; i++) {
            let tile = Index.squares[convert_index_to_coords(i)];
            for (let j = 0; j < tile.children[0].children.length; j++) {
                tile.children[0].children[i].className = tile.children[0].children[j].className.replaceAll(" on", "");
            }
        }
    }
}

//deprecated
//old function that used the path system
function dep_reset_maze(play_anim) {
    if (play_anim) {
        play_reset_animation((object) => {
            let children = object.children[0].children;
            for (let i = 0; i < children.length; i++) {
                for (let j = 0; j < children[i].children.length; j++) {
                    children[i].children[j].style.background = "var(--maze_bg)";
                }
            }
        });
    } else {
        //TODO: reset without anim
        for (let i = 0; i < 16*16; i++) {
            let tile = Index.squares[convert_index_to_coords(i)];
            let children = tile.children[0].children;
            for (let i = 0; i < children.length; i++) {
                for (let j = 0; j < children[i].children.length; j++) {
                    children[i].children[j].style.background = "var(--maze_bg)";
                }
            }
        }
    }
}

function play_reset_animation(on_finished = (obj) => {}) {
    let group_main = new AnimGroup(4);
    for (let i = 0; i < 16; i++) {
        let group_row = new AnimGroup(3);
        for (let j = 0; j < 16; j++) {
            let group_tile = new AnimGroup(-1);
            group_tile.add(new AnimBackgroundColor(25, Index.squares[[j, i]], "#0a0e1a", "#FFFFFF", on_finished));
            group_tile.add(new AnimBackgroundColor(25, Index.squares[[j, i]], "#FFFFFF", "#0a0e1a"));
            group_row.add(group_tile);
        }
        group_main.add(group_row);
    }

    Index.animHandler.add(group_main);
}

function convert_index_to_coords(i) {
    let x,y;
    x = (i)%16;
    y = (i-x)/16;
    return [x,y];
}

//just an alias function
function co_crds_i(i) {return convert_coords_to_index(i);}
function convert_coords_to_index(i) {
    return i[1]*16+i[0];
}

function select_square(square) {
    square.style.borderColor = "orange";

    let borderAnim = new AnimBorderInner(20, square, 3, 1, 2, [0, 34, 34]);
    Index.animHandler.addImmediate(borderAnim);

    current_state = ["point", "stopped"];
}

function unselect_square(square) {
    square.style.borderColor = "black";

    let borderAnim = new AnimBorderInner(20, square, 1, 3, 0, [2, 30, 30]);
    Index.animHandler.addImmediate(borderAnim);

    current_state = ["reset", "finished"];
}

function update_sensors(sensor, value) {
    let sensor_ele = document.getElementById(sensor);
    sensor_ele.innerHTML = (sensor === "left" ? "Links: " : (sensor === "front" ? "Vorne: " : "Rechts: "));
    sensor_ele.innerHTML = sensor_ele.innerHTML + value[0] + ":" + value[1];
}

function add_message(message) {
    let console_ele = document.getElementsByClassName("debug_console")[0];
    let message_ele = document.createElement("div");
    message_ele.className = "debug_console_message unselectable";
    message_ele.innerHTML = message;

    console_ele.appendChild(message_ele);

    let size = 0;
    for (let i = 0; i < console_ele.children.length; i++) {
        size += console_ele.children[i].clientHeight;
    }
    console_ele.scrollTop = console_ele.scrollHeight;
    /*console.log(size);
    if (size > 250) {
        console_ele.removeChild(console_ele.children.item(0));
    }*/
}

function updateControls(button_id, state) {
    let button = document.getElementById("button_" + button_id);
    if (state) {
        Index.animHandler.addImmediate(new AnimSlideColor(50, button, "var(--controll_disabled)", "var(--controll_enabled)"));
    } else {
        Index.animHandler.addImmediate(new AnimSlideColor(50, button, "var(--controll_enabled)", "var(--controll_disabled)"));
    }
}

function updateControls_disalbed() {
    let buttonStart = document.getElementsByClassName("button_start_stop")[0];
    let buttonPause = document.getElementsByClassName("button_pause")[0];
    let buttonReset = document.getElementsByClassName("button_reset")[0];

    if (current_state[1] === "finished") {
        buttonStart.style.backgroundImage = "linear-gradient(to right, darkgray, lightgray)";
        buttonPause.style.backgroundImage = "linear-gradient(to right, darkgray, lightgray)";
    } else if (current_state[1] === "stopped") {
        buttonStart.style.backgroundImage = "linear-gradient(to right, cyan, cornflowerblue)";
        buttonPause.style.backgroundImage = "linear-gradient(to right, darkgray, lightgray)";

        //buttonStart.style.animation = "animButtonAvailable 1s ease-in";
    } else if (current_state[1] === "running") {
        buttonPause.style.backgroundImage = "linear-gradient(to right, cyan, cornflowerblue)";
    }

    if (current_state[0] === "reset") {
        buttonReset.style.backgroundImage = "linear-gradient(to right, darkgray, lightgray)";
    } else if (current_state[0] === "point") {
        buttonReset.style.backgroundImage = "linear-gradient(to right, cyan, cornflowerblue)";
    }
}

function add_algorithm(algorithm) {
    let algorithm_ele = document.createElement("div");
    algorithm_ele.className = "pop_window_algorithm_choice unselectable";
    algorithm_ele.innerHTML = algorithm;
    algorithm_ele.addEventListener("click", function () {
        let request = new XMLHttpRequest();
        request.addEventListener("load", function () {console.log(this.responseText);});
        request.open("POST", document.location.origin + "/action");
        request.setRequestHeader("Content-Type", "Application/json");
        request.send(JSON.stringify({
            action: "algorithm_selected",
            algorithm: algorithm,
        }));
        Index.closePopAlgo();
    });
    document.getElementById("algo_content").appendChild(algorithm_ele);
}

function math_pos(x) {
    return x*(x<0?-1:1);
}