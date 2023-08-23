//! Contains native implementations of the network interface.
//!
//! This included quinn server+client and webtransport server using `h3`
pub mod client;
pub mod client_connection;
pub mod common;
pub mod server;
mod webtransport;

#[cfg(feature = "tls-native-roots")]
fn add_native_roots(roots: &mut rustls::RootCertStore) {
    tracing::debug!("Loading native root certificates");
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
}

#[cfg(feature = "tls-webpki-roots")]
fn add_webpki_roots(roots: &mut rustls::RootCertStore) {
    tracing::debug!("Loading webpki root certificates");
    roots.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));
}

fn load_root_certs() -> rustls::RootCertStore {
    #[cfg(any(feature = "tls-native-roots", feature = "tls-webpki-roots"))]
    {
        let mut roots = rustls::RootCertStore::empty();
        #[cfg(feature = "tls-native-roots")]
        add_native_roots(&mut roots);
        #[cfg(feature = "tls-webpki-roots")]
        add_webpki_roots(&mut roots);
        roots
    }
    #[cfg(not(any(feature = "tls-native-roots", feature = "tls-webpki-roots")))]
    {
        tracing::info!("Creating empty root certificates store");
        rustls::RootCertStore::empty()
    }
}
