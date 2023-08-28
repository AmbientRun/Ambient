use anyhow::Context;
use url::Url;

pub async fn read_file(url: &Url) -> anyhow::Result<String> {
    if url.scheme() == "file" {
        #[cfg(target_os = "unknown")]
        unimplemented!("file reading is not supported on web");

        #[cfg(not(target_os = "unknown"))]
        if let Ok(file_path) = url.to_file_path() {
            return Ok(std::fs::read_to_string(&file_path)
                .with_context(|| format!("failed to read path: {file_path:?}"))?);
        }
    }

    Ok(reqwest::get(url.clone())
        .await
        .with_context(|| format!("failed to get URL: {url:?}"))?
        .text()
        .await
        .with_context(|| format!("failed to get text from URL: {url:?}"))?)
}
