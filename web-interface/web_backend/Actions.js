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
}