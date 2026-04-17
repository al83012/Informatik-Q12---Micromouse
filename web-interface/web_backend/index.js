const { Action, Actions} = require('./Actions.js');

const { BackendManager } = require('./BackendManager.js');
const manager = new BackendManager();


//communication with backend
console.log("[B] Requiring Websocket");
const WebsocketClient = require('websocket').client;
const back_port = 8090;
const host = '127.0.0.1';
console.log("[B] Creating Client");
const client = new WebsocketClient();

function connect_backend()
{client.connect("ws://"+host+":"+back_port, "echo-protocol");};

client.on('connectFailed', (err) => {
    console.error('[B] Connect Error: ' + err.toString());
});

client.on('connect', (conn) => {
    console.log('[B] WebSocket Client Connected');
    manager.set_backend(client);
    manager.sync.push(Actions.update_con_status(true));

    conn.on('error', (err) => {
        console.log("[B] Implement ERROR");
        manager.set_backend(null);
        manager.sync.push(Actions.update_con_status(false));
        connect_backend();
    });
    conn.on('close', () => {
        console.log('[B] Connection Closed');
        manager.set_backend(null);
        manager.sync.push(Actions.update_con_status(false));
        connect_backend();
    });
    conn.on('message', (message) => {
        if (message.type === 'utf8') {
            manager.b_handleUpdate(JSON.parse(message.utf8Data));
        }
    });
});

console.log("[B] Connecting to Backend");
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
console.log("[L] Retrieving IP");
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
console.log("[F] Requiring Express");
const express = require('express');
const app = express();
const front_port = 80;


let actions = [];

console.log("[F] Creating Website host at: " + JSON.stringify(results));
app.use(express.static("./../web_frontend"));
app.use("/module", express.static("./../web_frontend/module"));
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
    res.redirect("/home.html");
})

app.listen(front_port, () => console.log(`[F] WebInterface listening on port ${front_port}!`));