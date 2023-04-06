use ambient_std::asset_url::ASSETS_PROTOCOL_SCHEME;

pub(crate) fn url(
    path: String,
) -> anyhow::Result<Option<String>> {
    Ok(Some(format!("{}:/{}", ASSETS_PROTOCOL_SCHEME, path.trim_start_matches('/'))))
}