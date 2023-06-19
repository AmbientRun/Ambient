import wasm from "vite-plugin-wasm";
import { defineConfig } from "vite";

// import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
    plugins: [wasm()],
    server: {
        headers: {
            // Enables shared array buffers
            "Access-Control-Allow-Origin": "*",
            "Cross-Origin-Opener-Policy": "same-origin",
            "Cross-Origin-Embedder-Policy": "require-corp",
        },
    },
});
