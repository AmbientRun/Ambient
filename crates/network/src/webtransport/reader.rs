use std::{
    borrow::Cow,
    io,
    marker::PhantomData,
    task::{Context, Poll},
};

use futures::{ready, FutureExt};
use js_sys::{Boolean, Reflect};
use thiserror::Error;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{ReadableStream, ReadableStreamDefaultReader};

pub struct StreamReader<T> {
    fut: Option<JsFuture>,
    _stream: ReadableStream,
    reader: ReadableStreamDefaultReader,
    marker: PhantomData<T>,
    label: Option<Cow<'static, str>>,
}

impl<T: JsCast> StreamReader<T> {
    pub async fn stop(&self) {
        JsFuture::from(self.reader.cancel())
            .await
            .expect("Failed to cancel stream");
    }

    pub fn new(label: Option<impl Into<Cow<'static, str>>>, stream: ReadableStream) -> Self {
        let reader = stream
            .get_reader()
            .dyn_into::<ReadableStreamDefaultReader>()
            .unwrap();
        Self {
            fut: None,
            _stream: stream,
            reader,
            marker: PhantomData,
            label: label.map(Into::into),
        }
    }
    pub fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<T, ReadError>>> {
        loop {
            if let Some(fut) = &mut self.fut {
                let chunk = ready!(fut.poll_unpin(cx)).map_err(|err| {
                    ReadError::ReadError(
                        self.label.clone(),
                        std::any::type_name::<T>(),
                        format!("{err:?}"),
                    )
                })?;
                self.fut = None;

                tracing::info!("Got: {chunk:?}");
                let done = Reflect::get(&chunk, &JsValue::from_str("done"))
                    .unwrap()
                    .dyn_into::<Boolean>()
                    .unwrap();

                if done.is_truthy() {
                    return Poll::Ready(None);
                } else {
                    let value = Reflect::get(&chunk, &JsValue::from_str("value"))
                        .unwrap()
                        .dyn_into::<T>()
                        .unwrap();

                    return Poll::Ready(Some(Ok(value)));
                }
            } else {
                let fut = JsFuture::from(self.reader.read());
                self.fut = Some(fut);
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("Failed to read from stream {0:?} {1}: {2:?}")]
    ReadError(Option<Cow<'static, str>>, &'static str, String),
}

impl From<ReadError> for io::Error {
    fn from(value: ReadError) -> Self {
        io::Error::new(io::ErrorKind::Other, value)
    }
}
