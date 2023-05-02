use std::str::RSplitTerminator;

use futures::{Sink, Stream};
use pin_project::pin_project;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::codec::FramedCodec;

/// Transport agnostic framed reader
#[pin_project]
pub struct RecvStream<T, S> {
    #[pin]
    read: FramedRead<S, FramedCodec<T>>,
}

impl<T, S> RecvStream<T, S>
where
    S: AsyncRead,
    T: serde::de::DeserializeOwned,
{
    pub fn new(stream: S) -> Self {
        Self { read: FramedRead::new(stream, FramedCodec::new()) }
    }
}

impl<T, S> Stream for RecvStream<T, S>
where
    S: AsyncRead,
    T: serde::de::DeserializeOwned,
{
    type Item = Result<T, bincode::Error>;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let p = self.project();

        p.read.poll_next(cx)
    }
}

/// Transport agnostic framed writer
#[pin_project]
pub struct SendStream<T, S> {
    #[pin]
    write: FramedWrite<S, FramedCodec<T>>,
}

impl<T, S> SendStream<T, S>
where
    S: AsyncWrite,
    T: serde::Serialize,
{
    pub fn new(stream: S) -> Self {
        Self { write: FramedWrite::new(stream, FramedCodec::new()) }
    }
}

impl<T, S> Sink<T> for SendStream<T, S>
where
    S: AsyncWrite,
    T: serde::Serialize,
{
    type Error = bincode::Error;

    fn poll_ready(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        let p = self.project();

        p.write.poll_ready(cx)
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        let p = self.project();

        p.write.start_send(item)
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        let p = self.project();

        p.write.poll_flush(cx)
    }

    fn poll_close(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        let p = self.project();

        p.write.poll_close(cx)
    }
}
