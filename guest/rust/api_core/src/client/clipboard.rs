use crate::internal::wit;

/// Get the current contents of the clipboard.
pub fn get() -> Option<String> {
    wit::client_clipboard::get()
}

/// Set the current contents of the clipboard.
pub fn set(text: &str) {
    wit::client_clipboard::set(text);
}
