use ambient_api::prelude::*;

#[cfg(feature = "server")]
mod server;

#[main]
pub async fn main() -> EventResult {
    #[cfg(feature = "server")]
    {
        server::main().await?;
    }

    EventOk
}
