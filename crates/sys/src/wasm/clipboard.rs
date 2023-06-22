use js_sys::JsString;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

/// Retrieves the contents of the clipboard.
pub async fn get() -> Option<String> {
    let text = JsFuture::from(
        web_sys::window()
            .expect("No window")
            .navigator()
            .clipboard()
            .expect("No clipboard")
            .read_text(),
    )
    .await
    .ok()?;

    Some(text.dyn_into::<JsString>().unwrap().into())
}

pub async fn set(text: &str) -> anyhow::Result<()> {
    JsFuture::from(
        web_sys::window()
            .expect("No window")
            .navigator()
            .clipboard()
            .expect("No clipboard")
            .write_text(text.into()),
    )
    .await
    .map_err(|v| anyhow::anyhow!("{:?}", v))
    .map(|_| ())
}
