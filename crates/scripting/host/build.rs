use std::path::Path;

fn main() {
    tilt_scripting_host_build::bundle_scripting_interface(
        Path::new("../guest/rust"),
        "tilt_base_scripting_interface.json",
        &["src", "interface_macros"],
    );
}
