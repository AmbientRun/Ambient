use anyhow::Context;
use openssl::hash::MessageDigest;
use std::process::Command;

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

    spawn(&spki, "http://localhost:5173")?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn detach_process(child: &mut Command) -> &mut Command {
    use std::os::unix::process::CommandExt;
    // Safety
    // Does not access any memory from *this* process.
    unsafe {
        child.pre_exec(|| {
            // Detach the child by moving it to a new process group
            if let Err(e) = nix::unistd::setsid() {
                // Safety: e is repr(i32) and it thus safe to format
                eprintln!("Failed to detach child {e}")
            };

            Ok(())
        })
    };

    // Prevent output from leaking into the parent terminal
    child
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
}

#[cfg(target_os = "macos")]
fn spawn(spki: &str, _url: &str) -> anyhow::Result<()> {
    let status = std::process::Command::new("open")
        // Feeding a url to chrome here makes `--args spki` not be fed to chrome
        .args([
            "-a",
            "Google Chrome",
            "--args",
            "--origin-to-force-quic-on=127.0.0.1:4433",
        ])
        .arg(format!("--ignore-certificate-errors-spki-list={spki}"))
        .spawn()
        .context("Failed to open Google Chrome")?
        .wait()
        .context("Failed to wait for launch command to exit")?;

    if !status.success() {
        anyhow::bail!("Failed to launch browser. Process exited with {status:?}");
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn spawn(spki: &str, url: &str) -> anyhow::Result<()> {
    let status = detach_process(
        std::process::Command::new("google-chrome")
            .arg(url)
            // Feeding a url to chrome here makes `--args spki` not be fed to chrome
            .arg(format!("--ignore-certificate-errors-spki-list={spki}")),
    )
    .spawn()
    .context("Failed to open Google Chrome")?
    .wait()
    .context("Failed to wait for launch command to exit")?;

    if !status.success() {
        anyhow::bail!("Failed to launch browser. Process exited with {status:?}");
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn spawn(_spki: &str, _url: &str) -> anyhow::Result<()> {
    anyhow::bail!("Launching the browser for windows is not yet supported.")
}
