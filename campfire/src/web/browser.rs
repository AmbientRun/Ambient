use std::time::Duration;

use anyhow::Context;
use base64::Engine;
use openssl::hash::MessageDigest;
use tokio::process::Command;

pub async fn open() -> anyhow::Result<()> {
    let cert_file = tokio::fs::read("./localhost.crt")
        .await
        .context("Failed to read certificate file")?;

    let der =
        openssl::x509::X509::from_der(&cert_file).context("Failed to deserialize certificate")?;

    let pubkey = der.public_key().context("Failed to get public key")?;

    let key_der = pubkey.public_key_to_der()?;
    let digest = openssl::hash::hash(MessageDigest::sha256(), &key_der)
        .context("Failed to produce digest of public key")?;

    let spki = openssl::base64::encode_block(&digest);

    eprintln!("Got SPKI: {:?}", &spki);

    open_browser(&spki, "http://localhost:5173").await?;

    Ok(())
}

async fn open_browser(spki: &str, url: &str) -> anyhow::Result<()> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            let mut command = Command::new("open");
            command
                // Feeding a url to chrome here makes `--args spki` not be fed to chrome
                .args(["-a", "Google Chrome", "--args"])
                .arg(format!("--ignore-certificate-errors-spki-list={spki}"));

        }
        else if #[cfg(target_os = "linux")]{
            let _spki = spki;
            let _url = url;
            let mut command = Command::new("google-chrome");
                command.args(["-a", "Google Chrome", url, "--args"])
                .arg(format!("--ignore-certificate-errors-spki-list={spki}"))
                .spawn()
                .context("Failed to spawn browser")?;

            anyhow::bail!("Launching the browser for linux is not supported. This is because cargo will cleanup the browser background process when campfire terminates")
        }
    }

    let status = command
        .spawn()
        .context("Failed to spawn browser")?
        .wait()
        .await
        .context("Failed to wait for launch command to exit")?;

    if !status.success() {
        anyhow::bail!("Failed to launch browser. Process exited with {status:?}");
    }

    Ok(())
}
