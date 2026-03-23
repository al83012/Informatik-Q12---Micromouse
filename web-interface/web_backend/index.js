const { Action, Actions} = require('./Actions.js');

const { BackendManager } = require('./BackendManager.js');
const manager = new BackendManager();


//communication with backend
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

client.connect( {port: back_port, host: host }, function () {
    console.log('Connected successfully to backend');
    manager.set_backend(client);
});

client.on('data', (data) => {
    manager.b_handleUpdate(data);
});


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