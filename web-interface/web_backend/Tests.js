const WebsocketClient  = require('websocket').client;
const backport = 8090;
const host = '127.0.0.1';
const timeout = 200;
const client = new WebsocketClient();

function connect_backend() {
    client.connect("ws://"+host+":"+backport);
}

client.on('connectFailed', (err) => {
    console.error('Connection Failed!');
    db(err);
});

client.on('error', (err) => {
    db('Error!');
    db(err);
})

const readline = require("node:readline");
const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

function db(message) {console.log(message);}

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

db("Tests:")
db("0: Send predetermined path change messages")

function input(string) {
    switch (string) {
        case "0":
            client.on('connect', (conn) => {
                conn.on('close', () => {
                    db("Test Stopped! Reason: Connection Closed");
                });

                conn.on('message', (message) => {
                    db("Received: |> " + message.utf8Data + " <|");
                });

                let obj1 = new Object(null);
                obj1.StrategyChange = new Object(null);
                obj1.StrategyChange.reset_map = false;

                let obj2 = JSON.parse(JSON.stringify(obj1));
                obj2.StrategyChange.set_position = new Object(null);
                obj2.StrategyChange.set_position.pos = new Object(null);
                obj2.StrategyChange.set_position.pos.x = 0;
                obj2.StrategyChange.set_position.pos.y = 0;
                obj2.StrategyChange.set_position.dir = "PosX";
                obj2.StrategyChange.set_strategy = null;
                obj2.StrategyChange.set_goal = null;
                obj2.reset_map = true;

                async function send_path_change() {
                    while (true) {
                        await sleep(timeout);
                        conn.sendUTF(JSON.stringify(obj1));
                        await sleep(timeout);
                        conn.sendUTF(JSON.stringify(obj2));
                    }
                };

                send_path_change()
            });
            break;
    }
}

rl.question("->", cmd => input(cmd));