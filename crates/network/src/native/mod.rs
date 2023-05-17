//! Contains native implementations of the network interface.
//!
//! This included quinn server+client and webtransport server using `h3`
pub mod client;
pub mod server;

#[tracing::instrument(level = "info")]
fn load_native_roots() -> rustls::RootCertStore {
    tracing::info!("Loading native roots");
    let mut roots = rustls::RootCertStore::empty();
    match rustls_native_certs::load_native_certs() {
        Ok(certs) => {
            for cert in certs {
                let cert = rustls::Certificate(cert.0);
                if let Err(e) = roots.add(&cert) {
                    tracing::error!(?cert, "Failed to parse trust anchor: {}", e);
                }
            }
        }

        Err(e) => {
            tracing::error!("Failed load any default trust roots: {}", e);
        }
    };

    roots
}
