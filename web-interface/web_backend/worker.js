const { parentPort, workerData } = require('node:worker_threads');

const manager = workerData.payload;

while (true) {
    console.log('Worker!');
}