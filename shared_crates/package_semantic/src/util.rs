use url::Url;

pub async fn read_file(url: &Url) -> anyhow::Result<String> {
    if let Ok(file_path) = url.to_file_path() {
        Ok(std::fs::read_to_string(file_path)?)
    } else {
        Ok(reqwest::get(url.clone()).await?.text().await?)
    }
}
