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

pub fn set_background(text: impl Into<String>, cb: impl 'static + FnOnce(anyhow::Result<()>)) {
    let text = text.into();
    tokio::task::block_in_place(|| cb(self::set_blocking(&text)))
}

pub fn get_background(cb: impl 'static + FnOnce(Option<String>)) {
    tokio::task::block_in_place(|| {
        let text = arboard::Clipboard::new()
            .ok()
            .and_then(|mut v| v.get_text().ok());

        cb(text);
    });
}
