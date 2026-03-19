const express = require('express');
const app = express();
const port = 8080;

const PkgActions = require('./Actions.js');

let actions = [];

app.use(express.static("./../web_frontend"));
app.use("/module", express.static("./../web_frontend/module"));

app.get('/update', (req, res) => {
    actions.push(new PkgActions.Action("new_path", new Map([["coords", '[1,1,2,2,3,3,4,4,5,5]']])));

    let result = '{"actions":[';

    for (let action of actions) {
        result += action.getString();
    }

    result += ']}';
    res.send(result);

    actions = [];
});

app.listen(port, () => console.log(`Example app listening on port ${port}!`));

//setup connection with backend_server