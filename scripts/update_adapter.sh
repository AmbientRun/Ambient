set -e

mkdir -p wasmtime

if [ -d tmp/wasmtime ]; then
    echo "pulling wasmtime"
    pushd tmp/wasmtime
    git pull
    git clean -fdx
    popd
else
    echo "cloning wasmtime"
    git clone https://github.com/bytecodealliance/wasmtime tmp/wasmtime --recursive
fi

pwd

DST=`realpath crates/wasm/wasi_snapshot_preview1.command.wasm`

pushd tmp/wasmtime

cargo build -p wasi-preview1-component-adapter --target wasm32-unknown-unknown --features command --no-default-features --release

cp -v target/wasm32-unknown-unknown/release/wasi_snapshot_preview1.wasm $DST

echo "Copied wasm adapter to $DST"

popd
