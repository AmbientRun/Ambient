pub(crate) mod component;
pub(crate) mod conversion;
pub(crate) mod executor;
pub(crate) mod generated;

#[allow(missing_docs)]
pub(crate) mod wit;

use crate::internal::executor::EXECUTOR;

use wit::{__link_section, exports, guest};
wit::export_bindings!(Guest);

extern "Rust" {
    fn main();
}

struct Guest;
impl guest::Guest for Guest {
    fn init() {
        once_cell::sync::Lazy::force(&EXECUTOR);
        unsafe { main() };
    }

    fn exec(source: guest::Source, message_name: String, message_data: Vec<u8>) {
        EXECUTOR.execute(source, message_name, message_data);
    }
}
