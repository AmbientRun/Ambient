#!/bin/node
const util = require('node:util');
const exec = util.promisify(require('node:child_process').exec);
const { exit } = require("process");

const samples = [
    ["guest/rust/examples/basics/async", 1],
    ["guest/rust/examples/basics/input", 1],
    ["guest/rust/examples/basics/image", 3],
    ["guest/rust/examples/basics/primitives", 1],
    ["guest/rust/examples/games/tictactoe", 1],
    ["guest/rust/examples/ui/raw_text", 1],
    ["guest/rust/examples/ui/text", 1],
    ["guest/rust/examples/ui/flow_layout", 1],
    ["guest/rust/examples/ui/rect", 1],
]

async function run() {
    let ok = true;
    for (const [path, seconds] of samples) {
        console.log(path, "running..");
        try {
            let res = await exec(`cargo run -- run ${path} --headless --screenshot-test ${seconds}`);
            console.log(path, "was ok");
        } catch (err) {
            console.log(path, 'Error:', err);
            ok = false;
        }
    }
    if (!ok) {
        console.log("Exiting with status code 1");
        exit(1);
    }
}
run();
