export class Action {
    constructor(action, data) {
        this.action = action;
        this.data = data;
    }

    getData() {
        let result = "";
        this.data.forEach((item, key) => {result += `"${key}":${item},`;});
        if (result.length > 0) {
            result += '||';
        }
        result = result.replace(',||', "");
        return result;
    }

    getString() {
        return `{"action":"${this.action}", "data":{${this.getData()}}}`;
    }
}

export class Actions {
    static toString(actions) {
        let result = '{"actions":[';
        for (let action of actions) {
            result += action.getString();
            result += ",";
        }
        if (result[result.length - 1] === ",") {result = result.substring(0, result.length - 1);}
        result += ']}';
        return result;
    }

    static new_path(coords) {
        let coords_string = "[";
        coords_string += coords[0];
        for (let i = 1; i < coords.length; i++) {
            coords_string += "," + coords[i].toString();
        }
        coords_string += "]";
        return new Action("new_path", new Map([["coords", coords_string]]));
    }

    static update_button(button_id, state) {
        return new Action("update_button", new Map([["button_id", button_id], ["state", state]]));
    }

    static add_message(message) {
        return new Action("add_message", new Map([["message", '"' + message + '"']]));
    }

    static update_sensor(sensor, values) {
        return new Action("update_sensor", new Map([["sensor", '"' + sensor + '"'], ["value", values]]));
    }

    static add_algorithm(algorithm) {
        return new Action("add_algorithm", new Map([["algorithm", '"' + algorithm.name + '"']]));
    }

    static update_algorithm(algorithm) {
        return new Action("update_algorithm", new Map([["algorithm", '"' + algorithm + '"']]));
    }

    static update_con_status(status) {
        return new Action("update_con_status", new Map([["status", status]]));
    }

    static reset_maze(play_anim) {
        return new Action("reset_maze", new Map([["animation", play_anim]]));
    }

    static update_path(path) {
        //TODO: convert path into string
        return new Action("update_path", new Map([["path", ""]]));
    }

    static move_mouse(x, y, x_new, y_new) {
        return new Action("move_mouse", new Map([["x", x], ["y", y], ["x_new", x_new], ["y_new", y_new]]));
    }

    static rotate_mouse(dir, dir_new) {
        let direction = dir;
        let direction_new = dir_new;
        if (typeof dir === "number") {
            switch (dir) {
                case 0:
                    direction = "n";
                    break;
                case 1:
                    direction = "e";
                    break;
                case 2:
                    direction = "s";
                    break;
                case 3:
                    direction = "w";
                    break;
            }
        }
        if (typeof dir_new === "number") {
            switch (dir_new) {
                case 0:
                    direction_new = "n";
                    break;
                case 1:
                    direction_new = "e";
                    break;
                case 2:
                    direction_new = "s";
                    break;
                case 3:
                    direction_new = "w";
                    break;
            }
        }
        return new Action("rotate_mouse", new Map([["dir", '"' + direction + '"'], ["dir_new", '"' + direction_new + '"']]));
    }

    static discover_tile(x, y, discovered_tiles, no_others) {
        let directions = [];
        let other_tiles = [];
        if (discovered_tiles.some(([a, b]) => x+1===a&&y===b)) {directions.push("e"); other_tiles.push([x+1, y]);}
        if (discovered_tiles.some(([a, b]) => x-1===a&&y===b)) {directions.push("w"); other_tiles.push([x-1, y]);}
        if (discovered_tiles.some(([a, b]) => x===a&&y-1===b)) {directions.push("n"); other_tiles.push([x, y-1]);}
        if (discovered_tiles.some(([a, b]) => x===a&&y+1===b)) {directions.push("s"); other_tiles.push([x, y+1]);}

        let dir_string = "[";
        for (let i = 0; i < directions.length; i++) {
            dir_string += '"' + directions[i] + '"';
            if (i < directions.length - 1) {dir_string += ",";}
        }
        dir_string += "]";

        let other_tiles_x = "[";
        let other_tiles_y = "[";
        let others = false;

        if (other_tiles.length > 0 && !no_others) {
            others = true;


            for (let i = 0; i < other_tiles.length; i++) {
                other_tiles_x += other_tiles[i][0];
                other_tiles_y += other_tiles[i][1];

                if (i < other_tiles.length - 1) {
                    other_tiles_x += ",";
                    other_tiles_y += ",";
                }
            }
        }

        other_tiles_x += "]";
        other_tiles_y += "]";

        return new Action("discover_tile", new Map([["x", x], ["y", y],
            ["directions", dir_string], ["others", others], ["other_tiles_x", other_tiles_x], ["other_tiles_y", other_tiles_y]]));
    }

    static show_loading() {
        return new Action("show_loading", new Map([]));
    }
    static hide_loading() {
        return new Action("hide_loading", new Map([]));
    }

    //Learned more JS, so I now use Objects for server
    // communication instead of whatever the Action System I coded was
    //Again trying to use objects, but forgot and coded discover tile :)
    static b_error(location, error, error_data) {
        let obj = new Object(null);
        obj.location = location;
        obj.error = [];
        obj.error[0] = error;
        for (let key in error_data) {
            obj.error.push(key);
        }
        return obj;
    }

    //TODO: change rotation for setposition
    static b_strategy_change(SetPosition = {is: false, x:0, y: 0, direction: "n"}, ResetMap = false, SetStrategy = {is: false, config: {name: "", config: {}}}, SetGoal = {is: false, x: 0, y: 0}) {
        let obj = new Object(null);
        obj.StrategyChange = new Object(null);
        if (SetPosition.is) {
            obj.StrategyChange.set_position = new Object(null);
            obj.StrategyChange.set_position.pos = new Object(null);
            obj.StrategyChange.set_position.pos.x = SetPosition.x;
            obj.StrategyChange.set_position.pos.y = SetPosition.y;
            switch (SetPosition.rotation) {
                case "n":
                    obj.StrategyChange.set_position.dir = "NegY";
                    break;
                case "s":
                    obj.StrategyChange.set_position.dir = "PosY";
                    break;
                case "w":
                    obj.StrategyChange.set_position.dir = "NegX";
                    break;
                case "o":
                    obj.StrategyChange.set_position.dir = "PosX";
                    break;
            }
        } else {
            obj.StrategyChange.set_position = null;
        }
        obj.StrategyChange.reset_map = ResetMap;

        if (SetStrategy.is) {
            obj.StrategyChange.set_strategy = new Object(null);
            obj.StrategyChange.set_strategy[SetStrategy.config.name] = new Object(null);
            for (var key in SetStrategy.config.config) {
                obj.StrategyChange.set_strategy[SetStrategy.config.name][key] = SetStrategy.config.config[key];
            }
        } else {
            obj.StrategyChange.set_strategy = null;
        }

        if (SetGoal.is) {
            obj.StrategyChange.set_goal = new Object(null);
            obj.StrategyChange.set_goal.x = SetGoal.x;
            obj.StrategyChange.set_goal.y = SetGoal.y;
        } else {
            obj.StrategyChange.set_goal = null;
        }

        return obj;
    }

    static b_pause() {
        let obj = new Object(null);
        obj = "Pause";

        return obj;
    }

    static b_continue() {
        let obj = new Object(null);
        obj = "Continue";

        return obj;
    }

    static b_test_strategychange() {
        let obj = new Object(null);
        obj.StrategyChange = new Object(null);
        obj.StrategyChange.reset_map = false;

        return obj;
    }

    static b_test() {
        let obj = new Object(null);
        obj.StrategyChange = new Object(null);

        obj.StrategyChange.set_position = new Object(null);
        obj.StrategyChange.set_position.pos = new Object(null);
        obj.StrategyChange.set_position.pos.x = 0;
        obj.StrategyChange.set_position.pos.y = 0;
        obj.StrategyChange.set_position.dir = "PosX";
        obj.StrategyChange.set_strategy = null;
        obj.StrategyChange.set_goal = null;
        obj.StrategyChange.reset_map = true;

        return obj;
    }


}