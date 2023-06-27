pub async fn get() -> Option<String> {
    arboard::Clipboard::new()
        .ok()
        .and_then(|mut v| v.get_text().ok())
}

pub async fn set(text: &str) -> anyhow::Result<()> {
    self::set_blocking(text)
}

pub fn set_blocking(text: &str) -> anyhow::Result<()> {
    arboard::Clipboard::new()?.set_text(text)?;

    Ok(())
}

pub fn set_background(text: impl Into<String>) {
    let text = text.into();
    if let Err(err) = self::set_blocking(&text) {
        tracing::error!("Failed to set clipboard: {:?}", err);
    }
}
