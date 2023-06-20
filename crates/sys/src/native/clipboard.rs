pub fn get() -> Option<String> {
    arboard::Clipboard::new()
        .ok()
        .and_then(|mut v| v.get_text().ok())
}
