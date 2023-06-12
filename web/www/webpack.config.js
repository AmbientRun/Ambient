const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
    entry: "./app.js",
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "app.js",
    },
    mode: "development",
    plugins: [
        new CopyWebpackPlugin(['index.html', "_headers", "processor.js"])
    ],
    devServer: {
        contentBase: path.join(__dirname, 'dist'),
        compress: true,
        port: 8080,
        headers: {
            "Access-Control-Allow-Origin": "*",
            "Cross-Origin-Opener-Policy": "same-origin",
            "Cross-Origin-Embedder-Policy": "require-corp"
        }
    }
};
