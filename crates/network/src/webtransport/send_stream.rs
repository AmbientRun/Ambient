use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{ready, FutureExt};
use js_sys::Uint8Array;
use tokio::io::AsyncWrite;
use wasm_bindgen_futures::JsFuture;
use web_sys::{WritableStream, WritableStreamDefaultWriter};

/// Allow sending a continuous stream bytes
pub struct SendStream {
    fut: Option<JsFuture>,
    stream: WritableStream,
    writer: Option<WritableStreamDefaultWriter>,
}

impl Drop for SendStream {
    fn drop(&mut self) {
        self.stop();
    }
}

impl From<SendError> for io::Error {
    fn from(value: SendError) -> Self {
        io::Error::new(io::ErrorKind::Other, value)
    }
}

impl SendStream {
    pub(crate) fn new(stream: WritableStream) -> Self {
        let writer = stream.get_writer().unwrap();

        SendStream {
            fut: None,
            stream,
            writer: Some(writer),
        }
    }

    /// Closes the sending stream
    pub fn stop(&mut self) {
        if let Some(writer) = self.writer.take() {
            writer.release_lock();
            let _ = self.stream.close();
        }
    }

    pub fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), SendError>> {
        if let Some(fut) = &mut self.fut {
            ready!(fut.poll_unpin(cx).map_err(|err| {
                tracing::error!("Sending failed: {err:?}");
                SendError::SendFailed(format!("{err:?}"))
            }))?;
            self.fut = None;

            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Ok(()))
        }
    }

    pub fn send_chunk(&mut self, buf: &[u8]) {
        if self.fut.is_some() {
            panic!("Send not ready");
        }

        let writer = self.writer.as_mut().expect("Stream is closed");

        let chunk = Uint8Array::from(buf);
        self.fut = Some(JsFuture::from(writer.write_with_chunk(&chunk)));
    }
}

impl AsyncWrite for SendStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        ready!(self.poll_ready(cx))?;

        let len = buf.len();
        self.send_chunk(buf);

        Poll::Ready(Ok(len))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.poll_ready(cx).map_err(Into::into)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Self::stop(&mut self);

        Poll::Ready(Ok(()))
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum SendError {
    #[error("Failed to send data to stream: {0}")]
    SendFailed(String),
    #[error("Failed to close the stream: {0}")]
    CloseFailed(String),
}
