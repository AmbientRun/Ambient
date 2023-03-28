use async_trait::async_trait;
use bytes::Bytes;
use quinn::{SendStream, RecvStream};

use crate::NetworkError;

#[async_trait]
pub trait Connection: Clone + Send + Sync {
    async fn open_uni(&self) -> Result<SendStream, NetworkError>;
    async fn open_bi(&self) -> Result<(SendStream, RecvStream), NetworkError>;
    async fn send_datagram(&self, data: Bytes) -> Result<(), NetworkError>;
}

#[async_trait]
impl Connection for quinn::Connection {
    async fn open_uni(&self) -> Result<SendStream, NetworkError> {
        Ok(self.open_uni().await?)
    }

    async fn open_bi(&self) -> Result<(SendStream, RecvStream), NetworkError> {
        Ok(self.open_bi().await?)
    }

    async fn send_datagram(&self, data: Bytes) -> Result<(), NetworkError> {
        Ok(self.send_datagram(data)?)
    }
}
