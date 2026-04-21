export class Action {
    constructor(action, data) {
        this.action = action;
        this.data = data;
    }

    getData() {
        let result = "";
        this.data.forEach((item, key) => {result += `"${key}":${item},`;});
        result += '||';
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
        return new Action("add_algorithm", new Map([["algorithm", '"' + algorithm + '"']]));
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

    static discover_tile(x, y, directions) {
        let dir_string = "[";
        for (let i = 0; i < directions.length; i++) {
            dir_string += '"' + directions[i] + '"';
            if (i < directions.length - 1) {dir_string += ",";}
        }
        dir_string += "]";
        return new Action("discover_tile", new Map([["x", x], ["y", y],
            ["directions", dir_string]]));
    }

    //Learned more JS, so I now use Objects for server
    // communication instead of whatever the Action System I coded was
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
}