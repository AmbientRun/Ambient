use anyhow::Context;
use sha2::{Digest, Sha256};
use spki::EncodePublicKey;
use std::process::Command;
use x509_certificate::X509Certificate;

pub async fn open() -> anyhow::Result<()> {
    let cert_file = tokio::fs::read("./localhost.crt")
        .await
        .context("Failed to read certificate file")?;

    let der = X509Certificate::from_der(cert_file).context("Failed to deserialize certificate")?;

    let pubkey = der.to_public_key_der().expect("Failed to get public key");

    let mut hasher = Sha256::new();
    hasher.update(pubkey.as_bytes());
    let digest = hasher.finalize();

    let spki = data_encoding::BASE64.encode(&digest);

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
    let status = Command::new("open")
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
    const CANDIDATES: &[&str] = &["google-chrome", "google-chrome-stable", "chromium"];

    let status = try_until_success(CANDIDATES, |cmd| {
        detach_process(
            Command::new(cmd)
                .arg(url)
                // Feeding a url to chrome here makes `--args spki` not be fed to chrome
                .arg(format!("--ignore-certificate-errors-spki-list={spki}")),
        )
        .spawn()
        .map_err(anyhow::Error::from)
    })
    .context("Failed to open Google Chrome; no installs found")?
    .wait()
    .context("Failed to wait for launch command to exit")?;

    if !status.success() {
        anyhow::bail!("Failed to launch browser. Process exited with {status:?}");
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn try_until_success<T>(
    inputs: &[&str],
    f: impl Fn(&str) -> anyhow::Result<T>,
) -> anyhow::Result<T> {
    let mut errors = vec![];

    for input in inputs {
        match f(input) {
            Ok(success) => return Ok(success),
            Err(error) => errors.push((input, error.to_string())),
        }
    }

    anyhow::bail!("{errors:?}")
}

#[cfg(target_os = "windows")]
fn spawn(spki: &str, url: &str) -> anyhow::Result<()> {
    // Chrome needs to be completely closed for arguments to be passed correctly
    let status = Command::new("cmd")
        .args(["/C", "start", "chrome", url])
        .arg(format!("--ignore-certificate-errors-spki-list={spki}"))
        .arg("--origin-to-force-quic-on=127.0.0.1:4433")
        .spawn()
        .context("Failed to open Google Chrome")?
        .wait()
        .context("Failed to wait for launch command to exit")?;

    if !status.success() {
        anyhow::bail!("Failed to launch browser. Process exited with {status:?}");
    }

    Ok(())
}
