#!/bin/node
const util = require('node:util');
const exec = util.promisify(require('node:child_process').exec);
const { exit } = require("process");
const { argv } = require('node:process');

let samples = [
    ["guest/rust/examples/basics/async", 30],
    ["guest/rust/examples/basics/input", 30],
    ["guest/rust/examples/basics/image", 30],
    ["guest/rust/examples/basics/transparency", 30],
    ["guest/rust/examples/basics/primitives", 30],
    ["guest/rust/examples/basics/raw_text", 30],
    ["guest/rust/examples/basics/fog", 60],
    ["guest/rust/examples/games/tictactoe", 30],
    ["guest/rust/examples/ui/button", 60],
    ["guest/rust/examples/ui/dock_layout", 60],
    ["guest/rust/examples/ui/editors", 60],
    ["guest/rust/examples/ui/flow_layout", 60],
    ["guest/rust/examples/ui/rect", 60],
    ["guest/rust/examples/ui/screens", 60],
    ["guest/rust/examples/ui/slider", 60],
    ["guest/rust/examples/ui/text", 60],
]

function process(nParallel, jobs) {
    return new Promise((resolve, reject) => {
        let running = 0;
        let index = 0;
        let results = [];
        function run() {
            if (index >= jobs.length) {
                if (running === 0) {
                    resolve(results);
                }
                return;
            }
            running++;
            let job = jobs[index++];
            job().then(result => {
                results.push(result);
                running--;
                run();
            }).catch(err => {
                reject(err);
            });
        }
        for (let i = 0; i < nParallel; i++) {
            run();
        }
    });
}

async function run(samples, just_build, no_build, nParallel) {
    console.time("time");
    let errors = (await process(nParallel, samples.map(([path, seconds], index) => async () => {
        console.timeLog("time", path, "running..");
        try {
            const command = just_build ? `build ${path}` : `run ${path} ${no_build ? '--no-build' : ''} --headless --no-proxy --golden-image-test ${seconds} --quic-interface-port ${9000 + index} --http-interface-port ${10000 + index}`;
            let fullCommand = `cargo run --release -- ${command}`;
            console.timeLog("time", path, fullCommand);
            let res = await exec(fullCommand);
            console.timeLog("time", path, "\x1b[32mwas ok\x1b[0m");
        } catch (err) {
            console.timeLog("time", path, "\x1b[31mfailed\x1b[0m");
            return { path, err };
        }
    }))).filter(x => x);
    for (const { path, err } of errors) {
        console.timeLog("time", `===================================== ERRORS FOR ${path} =====================================`);
        console.timeLog("time", err);
    }
    if (errors.length > 0) {
        console.timeLog("time", "Exiting with status code 1");
        exit(1);
    }
}
let just_build = false;
let no_build = false;
for (let i = 2; i < argv.length; i++) {
    if (argv[i] == "--build") {
        just_build = true;
    } else if (argv[i] == "--no-build") {
        no_build = true;
    } else {
        samples = samples.filter(([path]) => path.includes(argv[i]));
    }
}

run(samples, just_build, no_build, just_build ? 1 : 5);
