use std::{
    io, mem,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::{Buf, Bytes};
use futures::{future::poll_fn, ready, AsyncRead, AsyncWrite, Future, FutureExt};
use js_sys::Uint8Array;
use thiserror::Error;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    ReadableStream, ReadableStreamDefaultReader, WritableStream, WritableStreamDefaultWriter,
};

use crate::reader::{ReadError, StreamReader};
