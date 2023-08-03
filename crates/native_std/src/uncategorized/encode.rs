pub fn sha256_digest(value: &str) -> String {
    let digest = ring::digest::digest(&ring::digest::SHA256, value.as_bytes());
    data_encoding::HEXLOWER.encode(digest.as_ref())
}
