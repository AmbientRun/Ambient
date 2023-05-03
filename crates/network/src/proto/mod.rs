use crate::protocol::ServerInfo;

pub mod client;
pub mod server;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
/// Frame used by the client to send requests to the server
pub enum ServerControl {
    /// Connect to the server with the specified user id
    Connect(String),
    /// Client wants to disconnect
    Disconnect,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
/// Frame used by the server to send control information to the client
pub enum ClientControl {
    ServerInfo(ServerInfo),
    /// Graceful disconnect
    Disconnect,
}
