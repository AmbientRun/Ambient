window.ctx = new AudioContext();
ctx.suspend();

function URLFromFiles(files) {
    const promises = files.map((file) =>
        fetch(file).then((response) => response.text())
    );

    return Promise.all(promises).then((texts) => {
        const text = texts.join("");
        const blob = new Blob([text], { type: "application/javascript" });

        return URL.createObjectURL(blob);
    });
}

URLFromFiles(["processor.js"]).then((e) => {
    if (ctx.audioWorklet === undefined) {
        log("No AudioWorklet.");
    } else {
        ctx.audioWorklet.addModule(e).then(() => {
            window.dataSAB = new SharedArrayBuffer(2048 * 4); // 4 is the byte lenth
            window.pointerSAB = new SharedArrayBuffer(2 * 4);
            window.writePtr = new Uint32Array(window.pointerSAB, 0, 1);
            window.readPtr = new Uint32Array(window.pointerSAB, 4, 1);
            const n = new AudioWorkletNode(ctx, "processor", {
                processorOptions: {
                    dataSAB: window.dataSAB,
                    pointerSAB: window.pointerSAB,
                    writePtr: window.writePtr,
                    readPtr: window.readPtr,
                },
            });
            n.connect(ctx.destination);
        });
    }
});

window.audioStart = () => {
    ctx.resume();
}

window.audioStop = () => {
    ctx.suspend();
}

// A dependency graph that contains any wasm must all be imported
// asynchronously. This `bootstrap.js` file does the single async import, so
// that no one else needs to worry about it again.
import("ambient_web")
    .catch((e) => console.error("Error importing `ambient`:", e))
    .then((ambient) => {
        ambient.init_ambient(true, true);
        ambient.start();
    });
