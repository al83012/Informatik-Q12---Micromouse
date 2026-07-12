const fs = require("fs");

const history = [];

function record(vars, label = "") {
    const stack = new Error().stack.split("\n")[2];

    // Extract filename + line + column from stack trace
    const match = stack.match(/\((.*):(\d+):(\d+)\)/) ||
        stack.match(/at (.*):(\d+):(\d+)/);

    let location = {
        //file: "unknown",
        line: -1,
        //column: -1
    };

    if (match) {
        location = {
            //file: match[1],
            line: Number(match[2]),
            //column: Number(match[3])
        };
    }

    history.push({
        timestamp: Date.now(),
        label,
        ...location,
        variables: clone(vars)
    });
}


function clone(value) {
    try {
        return structuredClone(value);
    } catch {
        return String(value);
    }
}


function save(filename = "execution-trace.json") {
    fs.writeFileSync(
        "logs/" + filename,
        JSON.stringify(history, null, 2)
    );
}


module.exports = {
    record,
    save
};