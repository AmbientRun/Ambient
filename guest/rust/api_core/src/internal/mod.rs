pub(crate) mod component;
pub(crate) mod conversion;
pub(crate) mod executor;
pub(crate) mod generated;
pub(crate) mod wit;

use crate::internal::executor::EXECUTOR;

extern "Rust" {
    fn main();
}

struct Guest;
impl wit::guest::Guest for Guest {
    fn init() {
        once_cell::sync::Lazy::force(&EXECUTOR);
        unsafe { main() };
    }

    fn exec(source: wit::guest::Source, message_name: String, message_data: Vec<u8>) {
        EXECUTOR.execute(source, message_name, message_data);
    }
}

use wit::{__link_section, guest};
wit::export_bindings!(Guest);
