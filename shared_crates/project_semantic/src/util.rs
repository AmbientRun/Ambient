use url::Url;

pub fn ensure_url_is_directory(url: Url) -> Url {
    if url.as_str().ends_with('/') {
        url
    } else {
        let mut url = url.clone();
        url.set_path(&format!("{}/", url.path()));
        url
    }
}

pub async fn read_file(url: &Url) -> anyhow::Result<String> {
    if let Ok(file_path) = url.to_file_path() {
        Ok(std::fs::read_to_string(file_path)?)
    } else {
        Ok(reqwest::get(url.clone()).await?.text().await?)
    }
}
