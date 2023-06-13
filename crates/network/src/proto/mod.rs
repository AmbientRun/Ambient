use ambient_ecs::ExternalComponentDesc;
use ambient_std::asset_url::AbsAssetUrl;

pub mod client;
pub mod server;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
/// Request sent by the client to the server
pub enum ClientRequest {
    /// Connect to the server with the specified user id
    Connect(String),
    /// Client wants to disconnect
    Disconnect,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
/// Frame used by the server to send information to the client
pub enum ServerPush {
    ServerInfo(ServerInfo),
    /// Graceful disconnect
    Disconnect,
}

#[cfg(not(target_os = "unknown"))]
pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Miscellaneous information about the server that needs to be sent to the client during the handshake.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ServerInfo {
    /// The name of the project. Used by the client to figure out what to title its window. Defaults to "Ambient".
    pub project_name: String,

    // Base url of the content server.
    pub content_base_url: AbsAssetUrl,

    /// The version of the server. Used by the client to determine whether or not to keep connecting.
    /// Defaults to the version of the crate.
    /// TODO: use semver
    pub version: String,
    pub external_components: Vec<ExternalComponentDesc>,
}
