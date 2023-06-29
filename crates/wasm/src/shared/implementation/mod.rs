#[cfg(feature = "wit")]
pub mod asset;
#[cfg(feature = "wit")]
pub mod component;
#[cfg(feature = "wit")]
pub mod entity;
pub mod message;
#[cfg(feature = "wit")]
pub mod player;

pub fn unsupported<T>() -> anyhow::Result<T> {
    anyhow::bail!("This function is not supported on this side of the API. Please report this if you were able to access this function.")
}
