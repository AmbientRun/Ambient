use bytes::{BufMut, Bytes, BytesMut};
use futures::future::BoxFuture;
use tokio::io::AsyncWriteExt;

use crate::{client::ClientConnection, NetworkError, MAX_FRAME_SIZE};

impl ClientConnection for quinn::Connection {
    fn request_bi(&self, id: u32, data: Bytes) -> BoxFuture<Result<Bytes, NetworkError>> {
        Box::pin(async move {
            let (mut send, mut recv) = self.open_bi().await?;

            send.write_u32(id).await?;
            send.write_all(&data).await?;

            drop(send);

            let buf = recv.read_to_end(MAX_FRAME_SIZE).await?.into();

            Ok(buf)
        })
    }

    fn request_uni(&self, id: u32, data: Bytes) -> BoxFuture<Result<(), NetworkError>> {
        Box::pin(async move {
            let mut send = self.open_uni().await?;

            send.write_u32(id).await?;
            send.write_all(&data).await?;

            Ok(())
        })
    }

    fn send_datagram(&self, id: u32, data: Bytes) -> Result<(), NetworkError> {
        let mut bytes = bytes::BytesMut::with_capacity(4 + data.len());
        bytes.put_u32(id);
        bytes.put(data);

        self.send_datagram(bytes.freeze())?;

        Ok(())
    }
}

impl ClientConnection for crate::native::client_connection::ConnectionKind {
    fn request_bi(&self, id: u32, data: Bytes) -> BoxFuture<Result<Bytes, NetworkError>> {
        Box::pin(async move {
            let (mut send, mut recv) = self.open_bi().await?;

            send.write_u32(id).await?;
            send.write_all(&data).await?;

            drop(send);

            let buf = recv.read_to_end(MAX_FRAME_SIZE).await?.into();

            Ok(buf)
        })
    }

    fn request_uni(&self, id: u32, data: Bytes) -> BoxFuture<Result<(), NetworkError>> {
        Box::pin(async move {
            let mut send = self.open_uni().await?;

            send.write_u32(id).await?;
            send.write_all(&data).await?;

            Ok(())
        })
    }

    fn send_datagram(&self, id: u32, data: Bytes) -> Result<(), NetworkError> {
        let mut bytes = BytesMut::with_capacity(4 + data.len());
        bytes.put_u32(id);
        bytes.put(data);

        self.send_datagram(bytes.freeze())?;

        Ok(())
    }
}
