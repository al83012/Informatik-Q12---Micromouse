let is_square_selected = false;
let selected_square;
//first is current command like center/point/reset and second is the state of command like running/finished/stopped
let current_state = ["reset", "finished"];
let goal = [15, 15]; //the coords for going somewhere (cmd point)


import { Animation, AnimationHandler, AnimFadeIn, AnimFadeOut, AnimBorderOuter, AnimBorderInner, AnimGroup, AnimBorderColor } from "./Animation.js"
import { StyleAdder } from "./style_adder.js";

export class Index {
    static animHandler;

    static squares = {};

    static eventLoad() {
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

        updateControls();

        //creating the AnimationHandler
        Index.animHandler = new AnimationHandler();
        window.setInterval(() => {Index.animHandler.nextFrame();}, 10)

        //let borderAnim = new AnimBorderColor(100, Index.squares[[5, 5]], "darkblue", "cyan");
        //Index.animHandler.add(borderAnim);

        let request = new XMLHttpRequest();
        request.addEventListener("load", Index.handleUpdate);
        request.open("GET", document.location.origin + "/update_full");
        request.send();
    }

    static handleUpdate(response) {
        console.log(response.srcElement.response);
        let actions = JSON.parse(response.srcElement.response);
        actions["actions"].forEach(action => {
            let data = action["data"];
            switch (action["action"]) {
                case "change_button":
                    updateControls(data["button_id"], data["state"]);
                    break;
                case "add_message":
                    add_message(data["message"]);
                    break;
            }
        });
    }

    static buttonStartStop() {
        let pathGroup = new AnimGroup(5);

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

        Index.animHandler.add(pathGroup);

        let request = new XMLHttpRequest();
        request.addEventListener("load", function () {console.log(this.responseText);});
        request.open("GET", document.location.origin + "/update");
        request.send();
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
}

function convert_index_to_coords(i) {
    let x,y;
    x = (i)%16;
    y = (i-x)/16;
    return [x,y];
}

function select_square(square) {
    square.style.borderColor = "orange";

    let borderAnim = new AnimBorderInner(20, square, 3, 2, 2, [1, 30, 30]);
    Index.animHandler.addImmediate(borderAnim);

    current_state = ["point", "stopped"];
    updateControls();
}

function unselect_square(square) {
    square.style.borderColor = "black";

    let borderAnim = new AnimBorderInner(20, square, 2, 3, 1, [2, 28, 28]);
    Index.animHandler.addImmediate(borderAnim);

    current_state = ["reset", "finished"];
    updateControls();
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

function updateControls(button_id, state) {}

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