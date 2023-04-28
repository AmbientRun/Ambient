use crate::protocol::ServerInfo;

pub mod server;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
/// Sent by the client to the server to control or influence the connection
pub enum ClientControlFrame {
    /// Connect to the server with the specified user id
    Connect(String),
    /// Client wants to disconnect
    Disconnect,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ServerControlFrame {
    ServerInfo(ServerInfo),
    /// Graceful disconnect
    Disconnect,
}
