const { Action, Actions} = require('./Actions.js');

const { BackendManager } = require('./BackendManager.js');
const manager = new BackendManager();


//communication with backend
const WebsocketClient = require('websocket').client;
const back_port = 8090;
const host = '127.0.0.1';
const client = new WebsocketClient();

function connect_backend()
{client.connect("ws://"+host+":"+back_port, "maze-backend");};

client.on('connectFailed', (err) => {
    console.error('Connect Error: ' + err.toString());
});

client.on('connect', (conn) => {
    console.log('WebSocket Client Connected');
    manager.set_backend(client);
    manager.sync.push(Actions.update_con_status(true));

    conn.on('error', (err) => {
        console.log("Implement ERROR");
        manager.set_backend(null);
        manager.sync.push(Actions.update_con_status(false));
        connect_backend();
    });
    conn.on('close', () => {
        console.log('Connection Closed');
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


//communication with frontend
const express = require('express');
const app = express();
const front_port = 80;


let actions = [];

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

app.listen(front_port, () => console.log(`Example app listening on port ${front_port}!`));

//setup connection with backend_server