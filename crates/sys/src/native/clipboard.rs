pub fn get() -> Option<String> {
    arboard::Clipboard::new()
        .ok()
        .and_then(|mut v| v.get_text().ok())
}

pub fn set(text: &str) -> anyhow::Result<()> {
    arboard::Clipboard::new()?.set_text(text)?;

    Ok(())
}
