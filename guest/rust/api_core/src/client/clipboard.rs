use crate::{
    core::input::messages::ClipboardGet, internal::wit, prelude::wait_for_runtime_message,
};

/// Get the current contents of the clipboard.
pub async fn get() -> Option<String> {
    wit::client_clipboard::get();
    wait_for_runtime_message::<ClipboardGet>(|_| true)
        .await
        .contents
}

/// Set the current contents of the clipboard.
pub fn set(text: &str) {
    wit::client_clipboard::set(text);
}
