use std::io::Write;

use ambient_native_std::asset_cache::{AssetCache, SyncAssetKeyExt};
use ambient_settings::SettingsKey;

pub async fn handle(assets: &AssetCache) -> anyhow::Result<()> {
    let mut settings = SettingsKey.get(assets);
    if settings.general.api_token.is_some() {
        print!("There is already an API token specified in the settings file. Would you like to continue [y|n]? ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().to_ascii_lowercase().starts_with('y') {
            return Ok(());
        }
    }

    print!("Enter your API token: ");
    std::io::stdout().flush()?;

    let token = rpassword::read_password()?.trim().to_string();

    settings.general.api_token = Some(token);
    settings.write_to_file(None)?;
    // This isn't strictly speaking necessary as the application will immediately quit after this,
    // but you never know what kind of logic might run afterwards...
    SettingsKey.insert(assets, settings);

    Ok(())
}
