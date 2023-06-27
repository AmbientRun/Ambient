use bytes::{BufMut, Bytes, BytesMut};
use futures::future::BoxFuture;
use sec_http3::webtransport::server::WebTransportSession;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{client::ClientConnection, NetworkError, MAX_FRAME_SIZE};

impl ClientConnection for WebTransportSession<sec_http3_quinn::Connection, Bytes> {
    fn request_bi(&self, id: u32, data: Bytes) -> BoxFuture<Result<Bytes, NetworkError>> {
        Box::pin(async move {
            let (mut send, recv) = self.open_bi(self.session_id()).await?.split();

            // TODO: investigate concurrent send+recv
            send.write_u32(id).await?;
            send.write_all(&data).await?;

            drop(send);

            let mut buf = Vec::new();

            let read = recv
                .take(MAX_FRAME_SIZE as u64 + 1)
                .read_to_end(&mut buf)
                .await?;
            if read > MAX_FRAME_SIZE {
                return Err(NetworkError::FrameTooLarge);
            }

            Ok(buf.into())
        })
    }

    fn request_uni(&self, id: u32, data: Bytes) -> BoxFuture<Result<(), NetworkError>> {
        Box::pin(async move {
            let mut send = self.open_uni(self.session_id()).await?;

            send.write_u32(id).await?;
            send.write_all(&data).await?;

            Ok(())
        })
    }

    fn send_datagram(&self, id: u32, data: Bytes) -> BoxFuture<Result<(), NetworkError>> {
        let mut bytes = bytes::BytesMut::with_capacity(4 + data.len());
        bytes.put_u32(id);
        bytes.put(data);

        let res = self.send_datagram(bytes.freeze()).map_err(Into::into);

        Box::pin(futures::future::ready(res))
    }
}

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

    fn send_datagram(&self, id: u32, data: Bytes) -> BoxFuture<Result<(), NetworkError>> {
        let mut bytes = bytes::BytesMut::with_capacity(4 + data.len());
        bytes.put_u32(id);
        bytes.put(data);

        let res = self.send_datagram(bytes.freeze()).map_err(Into::into);

        Box::pin(futures::future::ready(res))
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

    fn send_datagram(&self, id: u32, data: Bytes) -> BoxFuture<Result<(), NetworkError>> {
        let mut bytes = BytesMut::with_capacity(4 + data.len());
        bytes.put_u32(id);
        bytes.put(data);

        let res = self.send_datagram(bytes.freeze()).map_err(Into::into);

        Box::pin(futures::future::ready(res))
    }
}
