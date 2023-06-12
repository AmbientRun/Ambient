use super::reader::{ReadError, StreamReader};
use bytes::{Buf, Bytes};
use futures::{ready, Future};
use js_sys::Uint8Array;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, ReadBuf};
use web_sys::ReadableStream;

/// A continuous stream of bytes
pub struct RecvStream {
    buf: Bytes,
    stream: StreamReader<Uint8Array>,
}

impl RecvStream {
    pub(crate) fn new(stream: ReadableStream) -> Self {
        Self {
            buf: Bytes::new(),
            stream: StreamReader::new(Some("recv_stream"), stream),
        }
    }

    /// Read the next chunk of data from the stream.
    pub fn poll_read_chunk(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<&mut Bytes, ReadError>>> {
        if self.buf.has_remaining() {
            return Poll::Ready(Some(Ok(&mut self.buf)));
        }

        let data = ready!(self.stream.poll_next(cx)?);
        if let Some(data) = data {
            self.buf = data.to_vec().into();
            Poll::Ready(Some(Ok(&mut self.buf)))
        } else {
            Poll::Ready(None)
        }
    }

    /// Read the next chunk of data from the stream.
    pub fn read_chunk(&mut self) -> ReadChunk {
        ReadChunk { stream: self }
    }
}

impl AsyncRead for RecvStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), io::Error>> {
        match ready!(self.poll_read_chunk(cx)) {
            Some(Ok(bytes)) => {
                let len = buf.remaining().min(bytes.len());

                buf.put_slice(&bytes[..len]);
                bytes.advance(len);

                Poll::Ready(Ok(()))
            }
            Some(Err(err)) => Poll::Ready(Err(err.into())),
            None => Poll::Ready(Ok(())),
        }
    }
}

/// Futures for reading the next chunk of bytes
pub struct ReadChunk<'a> {
    stream: &'a mut RecvStream,
}

impl<'a> Future for ReadChunk<'a> {
    type Output = Option<Result<Bytes, ReadError>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let data = ready!(self.stream.poll_read_chunk(cx)?);
        tracing::info!("Data: {data:?}");

        if let Some(data) = data {
            Poll::Ready(Some(Ok(std::mem::take(data))))
        } else {
            Poll::Ready(None)
        }
    }
}
