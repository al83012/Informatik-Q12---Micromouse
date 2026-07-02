const { Action, Actions} = require('./Actions.js');

const { BackendManager } = require('./BackendManager.js');
const manager = new BackendManager();

const { Worker } = require('node:worker_threads');
const { join, dirname } = require('node:path');
const worker_path = join(__dirname, 'worker.js');

//Worker doesn't work, idk why
function runWorker(workerData) {
    return new Promise((resolve, reject) => {
        let settled = false;

        const worker = new Worker(worker_path, {
            workerData,
            /*resourceLimits: {
                maxOldGenerationSizeMb: 64,
                maxYoungGenerationSizeMb: 16,
                stackSizeMb: 4
            }*/
        });

        /*const timeout = setTimeout(() => {
            if (settled) return;
            settled = true;
            worker.terminate();
            reject(new Error('Worker timeout'));
        }, 10_000);*/

        function safeResolve(value) {
            if (settled) return;
            settled = true;
            //clearTimeout(timeout);
            worker.terminate();
            resolve(value);
        }

        function safeReject(error) {
            if (settled) return;
            settled = true;
            //clearTimeout(timeout);
            worker.terminate();
            reject(error);
        }

        worker.once('message', (message) => {
            if (message.status === 'ok') {
                safeResolve(message.result);
            } else {
                safeReject(new Error(message.message));
            }
        });

        worker.once('error', (error) => {
            safeReject(error);
        });

        worker.once('exit', (code) => {
            if (code !== 0) {
                safeReject(new Error(`Worker exited with code ${code}`));
            }
        });
    });
}

async function loop_worker(manager) {
//    await runWorker(manager);
    new Worker(worker_path, {manager,});
}



class Options {
    static recon = false;
    static input = true;
}




/**Comment is deprecated -> moved to ".md"
 * Communication build with Backend
 * Sending:
 * type: type of command
 * values are concatenated after type
 * Example: {"type": "NewStrategy", "strategy_type": "<strategy type>"}
 *
 * Sending types: NewStrategy<strategy_type> ; Error<location, error[]>
 *     Error location: recv, send, backend
 *     Error error[]: [incorrect_data, (type || data)], [missing_data, <location>], [unknown_destination]
 *     Error is sent on next possible occasion;
 *
 * Receiving:
 * type: type of command
 */
//communication with backend
console.log("\x1b[33m[B] Requiring Gateway");
const { gateway4async, gateway4sync } = require('default-gateway');
let gateway, version, int;
function find_gateway() {
    const {gateway_f, version_f, int_f} = gateway4sync();
    gateway = gateway_f;
    version = version_f;
    int = int_f;
};
const {gateway_f, version_f, int_f} = gateway4sync();
console.log("\x1b[33m[B] Found Gateway on: " + gateway_f + "; version: " + version_f + "; int: " + int_f + "");

console.log("\x1b[33m[B] Requiring Websocket");
const WebsocketClient = require('websocket').client;
const back_port = 8090;
const host = '127.0.0.1'//'192.168.137.1';
console.log("\x1b[33m[B] Creating Client");
const client = new WebsocketClient();

function connect_backend() {
    console.log("\x1b[33m[B] Attempting Connection");
    client.connect("ws://"+host+":"+back_port);
};

client.on('connectFailed', (err) => {
    console.error('\x1b[33m[B] \x1b[31mConnect Error: ' + err.toString() + '\x1b[33m');
    if (Options.recon) {
        setTimeout(connect_backend, 500);
    }
});

client.on('error', (err) => {
    console.log('\x1b[33m[B] \x1b[31mCritical Error: ' + err.toString());
})

//loop_worker(manager); //cant run because of infinite loop in worker

client.on('connect', (conn) => {

    console.log('\x1b[33m[B] WebSocket Client Connected');
    manager.set_backend(conn);
    manager.backend_client = client;
    manager.f_sync.push(Actions.update_con_status(true));

    manager.b_sync(Actions.b_test());

    conn.on('error', (err) => {
        console.log("\x1b[33m[B] \x1b[31mImplement ERROR");
        manager.set_backend(null);
        manager.f_sync.push(Actions.update_con_status(false));
        connect_backend();
    });
    conn.on('close', () => {
        console.log('\x1b[33m[B] Connection Closed');
        manager.set_backend(null);
        manager.f_sync.push(Actions.update_con_status(false));
        connect_backend();
    });
    conn.on('message', (message) => {
        if (message.type === 'utf8') {
            manager.b_handlePost(JSON.parse(message.utf8Data));
            //console.log(message.utf8Data);
        }
    });
});

console.log("\x1b[33m[B] Connecting to Backend");
connect_backend();

/*old deprecated due to protocol
const net = require('net');
const back_port = 8090; //which is from arne?
const host = '127.0.0.1';
const client = new net.Socket();

client.on('error', (err) => {console.log("Implement ERROR")})

/*client.on('error', (err) => {
    console.error(err);
    client.connect({port: back_port, host: host }, function () {
        console.log('Reconnected successfully');
    });
});*/ //for actual production
/*
client.connect( {port: back_port, host: host }, function () {
    console.log('Connected successfully to backend');
    manager.set_backend(client);
});

client.on('data', (data) => {
    manager.b_handleUpdate(data);
});*/

//retrieving lokal ip
console.log("\x1b[34m[L] Retrieving IP");
const { networkInterfaces } = require('os');

const nets = networkInterfaces();
const results = Object.create(null); // Or just '{}', an empty object

for (const name of Object.keys(nets)) {
    for (const net of nets[name]) {
        // Skip over non-IPv4 and internal (i.e. 127.0.0.1) addresses
        // 'IPv4' is in Node <= 17, from 18 it's a number 4 or 6
        const familyV4Value = typeof net.family === 'string' ? 'IPv4' : 4
        if (net.family === familyV4Value && !net.internal) {
            if (!results[name]) {
                results[name] = [];
            }
            results[name].push(net.address);
        }
    }
}


//communication with frontend
console.log("\x1b[32m[F] Requiring Express");
const express = require('express');
const app = express();
const front_port = 3000;


let actions = [];

console.log("\x1b[32m[F] Creating Website host at: " + JSON.stringify(results));
app.use(express.static("./../web_frontend"));
app.use("/module", express.static("./../web_frontend/module"));
app.use("/favicon.ico", express.static("./../web_frontend/favicon.ico"));
app.use(express.json());

app.get('/update', (req, res) => {
    manager.f_handleUpdate(res);
});

app.get('/update_full', (req, res) => {
    res.send(Actions.toString(manager.get_full()));
})

app.post('/action', (req, res) => {
    manager.f_handlePost(req.body);
    res.send("handled");
});

app.get('/', (req, res) => {
    res.redirect("/home_new.html");
});

app.post('/error', (req, res) => {
    manager.f_handleError(req.body);
    res.send("handled");
});

app.listen(front_port, () => console.log(`\x1b[32m[F] WebInterface listening on port ${front_port}!`));


const readline = require("node:readline")
const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

const { Utils } = require('./Utils.ts');

function input(string) {
    let parts = string.split(":");
    switch (parts[0]) {
        case "norecon":
            Options.recon = false;
            break;
        case "b":
            if (parts.length >= 2) {
                switch (parts[1]) {
                    case "recon":
                        Options.recon = !Options.recon;
                        if (Options.recon) {
                            connect_backend();
                        }
                        break;
                    case "send":
                        if (parts.length >= 3) {
                            try {
                                if (parts.length === 4) {
                                    manager.b_sync(Actions[parts[2]](...parts[3].split(",")));
                                } else {
                                    manager.b_sync(Actions[parts[2]]());
                                }
                            } catch (e) {
                                console.log(e);
                            }
                        }
                        break;
                }
            }
            break;
        case "f":
            if (parts.length >= 2) {
                switch (parts[1]) {
                    case "load":
                        if (parts.length === 3) {
                            if (parts[2] === "show") {
                                manager.f_sync.push(Actions.show_loading());
                            } else if (parts[2] === "hide") {
                                manager.f_sync.push(Actions.hide_loading());
                            }
                        }
                        break;
                    case "console":
                        if (parts.length === 3) {
                            manager.f_sync.push(Actions.add_message(parts[2]));
                        }
                        break;
                    case "send":
                        if (parts.length === 4) {
                            manager.f_sync.push(Actions[parts[2]](manager.in_maze.discovered,...parts[3].split(",")));
                        }
                        break;

                }
            }
            break;
        default:
            Utils.is(parts, "f:path:create:random", (rest) => {
                /*let path = [];

                let count = Math.floor(Math.random() * 5) + 1;

                for (;count > 0; count--) {
                    let dir = Math.floor(Math.random() * 2);
                    let amount = Math.floor(Math.random() * 10) + 1;
                    if (dir === 0) {
                        if (path.length > 0) {
                            if (path[path.length - 1][path[path.length-1].length-1] === "") {//TODO: complete test path generation with walker?

                            }
                        }
                    } else if (dir === 1) {

                    }
                }*/

                const target = { x: 8, y: 8 };

                const path = [{ x: 0, y: 0 }];
                const visited = new Set(["0,0"]);

                while (path.at(-1).x !== 8 || path.at(-1).y !== 8) {
                    const { x, y } = path.at(-1);

                    const moves = [
                        [1, 0], [-1, 0], [0, 1], [0, -1]
                    ]
                        .map(([dx, dy]) => ({ x: x + dx, y: y + dy }))
                        .filter(p =>
                            p.x >= 0 && p.x < 16 &&
                            p.y >= 0 && p.y < 16 &&
                            !visited.has(`${p.x},${p.y}`)
                        )
                        .sort(() => Math.random() - 0.5)
                        .sort((a, b) =>
                            Math.abs(a.x - target.x) + Math.abs(a.y - target.y) -
                            Math.abs(b.x - target.x) - Math.abs(b.y - target.y)
                        );

                    if (moves.length) {
                        path.push(moves[0]);
                        visited.add(`${moves[0].x},${moves[0].y}`);
                    } else {
                        visited.delete(`${x},${y}`); //run into a non-solvable state
                        path.pop();
                    }
                }


            });
            break;
    }

    setTimeout(con_in, 1);
}

async function con_in() {
    rl.question("->", cmd => input(cmd));
}

con_in()