import "./style.css";

console.info("Importing ambient web assembly module");

// A dependency graph that contains any wasm must all be imported
// asynchronously. This `bootstrap.js` file does the single async import, so
// that no one else needs to worry about it again.
import("ambient_web")
    .catch((e) => console.error("Error importing `ambient`:", e))
    .then((ambient) => {
        if (!ambient) {
            console.error("Ambient is null");
            return;
        }

        ambient.init_ambient(true, true);
        ambient.start();
    });
