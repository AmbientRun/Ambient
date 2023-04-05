pub mod component;
pub mod entity;
pub mod event;
pub mod message;
pub mod asset;

pub fn unsupported<T>() -> anyhow::Result<T> {
    anyhow::bail!("This function is not supported on this side of the API. Please report this if you were able to access this function.")
}
