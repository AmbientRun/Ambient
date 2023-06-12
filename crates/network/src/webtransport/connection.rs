use std::{
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::anyhow;
use bytes::Bytes;
use futures::{ready, Future};
use js_sys::Uint8Array;
use parking_lot::Mutex;
use url::Url;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    ReadableStream, WebTransport, WebTransportBidirectionalStream, WritableStream,
    WritableStreamDefaultWriter,
};

use crate::NetworkError;

use super::{
    reader::{ReadError, StreamReader},
    RecvStream, SendStream,
};

/// The webtransport connection
///
/// Disconnects when dropped
pub struct Connection {
    transport: WebTransport,
    datagrams: WritableStreamDefaultWriter,
    incoming_datagrams: Mutex<StreamReader<Uint8Array>>,
    incoming_recv_streams: Mutex<StreamReader<ReadableStream>>,
    incoming_bi_streams: Mutex<StreamReader<WebTransportBidirectionalStream>>,
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.transport.close();
    }
}

impl Connection {
    /// Open a connection to `url`
    pub async fn connect(url: Url) -> anyhow::Result<Self> {
        let transport = WebTransport::new(url.as_str()).map_err(|e| anyhow!("{e:?}"))?;

        JsFuture::from(transport.ready())
            .await
            .map_err(|e| anyhow!("{e:?}"))?;

        tracing::info!("Connection ready");

        let datagrams = transport.datagrams();
        let datagrams = datagrams.writable().get_writer().unwrap();
        let incoming_datagrams = transport.datagrams().readable();

        let incoming_datagrams = Mutex::new(StreamReader::new(
            Some("incoming_datagrams"),
            incoming_datagrams,
        ));

        let incoming_recv_streams = Mutex::new(StreamReader::new(
            Some("incoming_uni"),
            transport.incoming_unidirectional_streams(),
        ));

        let incoming_bi_streams = Mutex::new(StreamReader::new(
            Some("incoming_bi"),
            transport.incoming_bidirectional_streams(),
        ));

        Ok(Connection {
            transport,
            datagrams,
            incoming_datagrams,
            incoming_recv_streams,
            incoming_bi_streams,
        })
    }

    pub async fn open_uni(&self) -> Result<SendStream, NetworkError> {
        let stream = JsFuture::from(self.transport.create_unidirectional_stream())
            .await
            .map_err(|_| NetworkError::ConnectionClosed)?
            .dyn_into::<WritableStream>()
            .unwrap();

        Ok(SendStream::new(stream))
    }

    pub async fn open_bi(&self) -> Result<(SendStream, RecvStream), NetworkError> {
        let stream = JsFuture::from(self.transport.create_bidirectional_stream())
            .await
            .map_err(|_| NetworkError::ConnectionClosed)?
            .dyn_into::<WebTransportBidirectionalStream>()
            .unwrap();

        let send = stream.writable().dyn_into().unwrap();
        let recv = stream.readable().dyn_into().unwrap();

        Ok((SendStream::new(send), RecvStream::new(recv)))
    }

    /// Accepts an incoming bidirectional stream
    pub fn accept_bi(&self) -> AcceptBi {
        AcceptBi {
            stream: &self.incoming_bi_streams,
        }
    }

    /// Accepts an incoming unidirectional stream
    pub fn accept_uni(&self) -> AcceptUni {
        AcceptUni {
            stream: &self.incoming_recv_streams,
        }
    }

    /// Reads the next datagram from the connection
    pub fn read_datagram(&self) -> ReadDatagram<'_> {
        ReadDatagram {
            stream: &self.incoming_datagrams,
        }
    }

    /// Sends data to a WebTransport connection.
    pub fn send_datagram(&self, data: &[u8]) -> impl Future<Output = Result<(), NetworkError>> {
        let data = Uint8Array::from(data);
        let fut = JsFuture::from(self.datagrams.write_with_chunk(&data));

        async move {
            let _stream = fut.await.map_err(|_| NetworkError::ConnectionClosed)?;
            tracing::info!("Sent datagram {_stream:?}");
            Ok(())
        }
    }
}

/// Reads the next datagram from the connection
pub struct ReadDatagram<'a> {
    stream: &'a Mutex<StreamReader<Uint8Array>>,
}

impl Future for ReadDatagram<'_> {
    type Output = Option<Result<Bytes, ReadError>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut datagrams = self.stream.lock();

        let data = ready!(datagrams.poll_next(cx));

        match data {
            Some(Ok(data)) => Poll::Ready(Some(Ok(data.to_vec().into()))),
            Some(Err(err)) => Poll::Ready(Some(Err(err))),
            None => Poll::Ready(None),
        }
    }
}

/// Reads the next datagram from the connection
pub struct AcceptUni<'a> {
    stream: &'a Mutex<StreamReader<ReadableStream>>,
}

impl<'a> Future for AcceptUni<'a> {
    type Output = Option<Result<RecvStream, ReadError>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut datagrams = self.stream.lock();

        let data = ready!(datagrams.poll_next(cx)?);

        match data {
            Some(data) => Poll::Ready(Some(Ok(RecvStream::new(data)))),
            None => Poll::Ready(None),
        }
    }
}

/// Reads the next datagram from the connection
pub struct AcceptBi<'a> {
    stream: &'a Mutex<StreamReader<WebTransportBidirectionalStream>>,
}

impl<'a> Future for AcceptBi<'a> {
    type Output = Option<Result<(SendStream, RecvStream), ReadError>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut datagrams = self.stream.lock();

        let data = ready!(datagrams.poll_next(cx)?);

        match data {
            Some(data) => {
                let send = data.writable().dyn_into().unwrap();
                let recv = data.readable().dyn_into().unwrap();

                Poll::Ready(Some(Ok((SendStream::new(send), RecvStream::new(recv)))))
            }
            None => Poll::Ready(None),
        }
    }
}
