use anyhow::Context;
use base64::Engine;
use openssl::hash::MessageDigest;

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
    let status = tokio::process::Command::new("open")
        .args(["-a", "Google Chrome", url, "--args"])
        .arg(format!("--ignore-certificate-errors-spki-list={spki}"))
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
