use std::{io, path::Path};

pub async fn read(_path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "File IO on wasm it not supported",
    ))
}

pub async fn read_to_string(_path: impl AsRef<Path>) -> io::Result<String> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "File IO on wasm it not supported",
    ))
}

pub async fn write(_path: impl AsRef<Path>, _contents: impl AsRef<[u8]>) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "File IO on wasm it not supported",
    ))
}
