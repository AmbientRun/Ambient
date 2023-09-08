const fs = require('fs');
const path = require('path');
const launches = [];
for (const dir of fs.readdirSync('../examples/')) {
    // check if dir is a directory
    if (fs.lstatSync(path.join('../examples/', dir)).isDirectory()) {
        // loop through sub directories
        for (const subDir of fs.readdirSync(path.join('../examples/', dir))) {
            // check if subDir is a directory
            if (fs.lstatSync(path.join('../examples/', dir, subDir)).isDirectory()) {
                console.log(dir, subDir);
                launches.push({
                    "name": `${dir}/${subDir}`,
                    "type": "lldb",
                    "request": "launch",
                    "program": "ambient",
                    "args": [
                        "run",
                        "--debugger"
                    ],
                    "initCommands": [
                        "settings set plugin.jit-loader.gdb.enable on"
                    ],
                    "cwd": `\${workspaceFolder}/examples/${dir}/${subDir}`,
                })
            }
        }
    }
}
fs.writeFileSync('./launch.json', JSON.stringify({
    "version": "0.2.0",
    "configurations": launches
}, null, 4))
