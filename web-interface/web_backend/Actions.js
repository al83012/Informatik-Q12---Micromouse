export class Action {
    constructor(action, data) {
        this.action = action;
        this.data = data;
    }

    getData() {
        let result = "";
        this.data.forEach((item, key) => {result += `"${key}":${item},`;});
        result += '"';
        result = result.replaceAll(',"', "");
        return result;
    }

    getString() {
        return `{"action":"${this.action}", "data":{${this.getData()}}}`;
    }
}