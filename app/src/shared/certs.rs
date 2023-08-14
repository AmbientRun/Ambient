#[cfg(not(feature = "no_bundled_certs"))]
pub const CERT: &[u8] = include_bytes!("../../../localhost.crt");

#[cfg(not(feature = "no_bundled_certs"))]
pub const CERT_KEY: &[u8] = include_bytes!("../../../localhost.key");
