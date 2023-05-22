use ambient_web::{init_ambient, start};

fn main() {
    init_ambient(true, true);
    wasm_bindgen_futures::spawn_local(start(None));
}
