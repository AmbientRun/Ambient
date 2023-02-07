use std::path::Path;

fn main() {
    elements_scripting_host_build::bundle_scripting_interface(
        Path::new("../guest/rust"),
        "elements_runtime_scripting_interface.json",
        &["src"],
    );
}
