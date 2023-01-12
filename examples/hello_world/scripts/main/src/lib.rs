use elements_scripting_interface::*;

pub async fn main() -> EventResult {
    loop {
        println!("Hello, world! It is {}", time());
        sleep(0.5).await;
    }

    EventOk
}

#[no_mangle]
pub extern "C" fn call_main(runtime_interface_version: u32) {
    if INTERFACE_VERSION != runtime_interface_version {
        panic!("This script was compiled with interface version {{INTERFACE_VERSION}}, but the script host is running with version {{runtime_interface_version}}");
    }
    run_async(main());
}
