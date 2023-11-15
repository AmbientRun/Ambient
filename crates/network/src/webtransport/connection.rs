use std::{
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::{anyhow, Context as _};
use bytes::Bytes;
use futures::{ready, Future};
use js_sys::{Function, Reflect, Uint8Array};
use parking_lot::Mutex;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    ReadableStream, WebTransport, WebTransportBidirectionalStream, Worker, WritableStream,
};

use crate::NetworkError;

use super::{
    reader::{ReadError, StreamReader},
    recv_stream, RecvStream, SendStream,
};

enum WorkerRequest<'a> {
    Connect(&'a str),
    PollDatagrams,
    SendDatagram(&'a [u8]),
    OpenUni,
    AcceptUni,
    AcceptBi,
}

impl<'a> Into<JsValue> for WorkerRequest<'a> {
    fn into(self) -> JsValue {
        match self {
            Self::Connect(url) => {
                let array = js_sys::Array::new_with_length(2);
                array.set(0, JsValue::from("connect")); // TODO: turn to ints
                array.set(1, JsValue::from(url));
                array.into()
            }
            Self::PollDatagrams => {
                let array = js_sys::Array::new_with_length(1);
                array.set(0, JsValue::from("poll_datagrams")); // TODO: turn to ints
                array.into()
            }
            Self::SendDatagram(data) => {
                let array = js_sys::Array::new_with_length(2);
                array.set(0, JsValue::from("send_datagram")); // TODO: turn to ints
                array.set(1, Uint8Array::from(data).into());
                array.into()
            }
            Self::OpenUni => {
                let array = js_sys::Array::new_with_length(1);
                array.set(0, JsValue::from("open_uni")); // TODO: turn to ints
                array.into()
            }
            Self::AcceptUni => {
                let array = js_sys::Array::new_with_length(1);
                array.set(0, JsValue::from("accept_uni")); // TODO: turn to ints
                array.into()
            }
            Self::AcceptBi => {
                let array = js_sys::Array::new_with_length(1);
                array.set(0, JsValue::from("accept_bi")); // TODO: turn to ints
                array.into()
            }
        }
    }
}

enum WorkerResponse {
    Ready,
    ConnectError(String),
    Datagram(Option<Bytes>),
    OpenedUni(Result<SendStream, String>),
    AcceptedUni(Option<Result<RecvStream, String>>),
    AcceptedBi(Option<Result<(SendStream, RecvStream), String>>),
}

impl TryFrom<JsValue> for WorkerResponse {
    type Error = &'static str;

    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        let Ok(array) = value.dyn_into::<js_sys::Array>() else {
            return Err("not an array");
        };
        let Some(resp) = array.get(0).as_string() else {
            return Err("missing response");
        };
        match resp.as_str() {
            "ready" => Ok(Self::Ready),
            "connect_error" => {
                let err = array.get(1).as_string().unwrap();
                Ok(Self::ConnectError(err))
            }
            "datagram" => {
                let val = array.get(1);
                if val.is_null() {
                    Ok(Self::Datagram(None))
                } else {
                    let data = val.dyn_into::<Uint8Array>().unwrap();
                    Ok(Self::Datagram(Some(data.to_vec().into())))
                }
            }
            "opened_uni" => {
                let result = match array.get(1).dyn_into::<WritableStream>() {
                    Ok(stream) => Ok(SendStream::new(stream)),
                    Err(err) => Err(format!("{:?}", err)),
                };
                Ok(Self::OpenedUni(result))
            }
            "accepted_uni" => {
                let val = array.get(1);
                if val.is_null() {
                    Ok(Self::AcceptedUni(None))
                } else {
                    let result = match val.dyn_into::<ReadableStream>() {
                        Ok(stream) => Ok(RecvStream::new(stream)),
                        Err(err) => Err(format!("{:?}", err)),
                    };
                    Ok(Self::AcceptedUni(Some(result)))
                }
            }
            "accepted_bi" => {
                let val = array.get(1);
                if val.is_null() {
                    Ok(Self::AcceptedBi(None))
                } else {
                    let send_stream = match val.dyn_into::<WritableStream>() {
                        Ok(stream) => SendStream::new(stream),
                        Err(err) => return Ok(Self::AcceptedBi(Some(Err(format!("{:?}", err))))),
                    };
                    let recv_stream = match array.get(2).dyn_into::<ReadableStream>() {
                        Ok(stream) => RecvStream::new(stream),
                        Err(err) => return Ok(Self::AcceptedBi(Some(Err(format!("{:?}", err))))),
                    };
                    Ok(Self::AcceptedBi(Some(Ok((send_stream, recv_stream)))))
                }
            }
            _ => Err("unknown response"),
        }
    }
}

/// The webtransport connection
///
/// Disconnects when dropped
pub struct Connection {
    worker: Worker,
    incoming_datagrams: flume::Receiver<Option<Bytes>>,
    incoming_recv_streams: flume::Receiver<Option<Result<RecvStream, String>>>,
    incoming_bi_streams: flume::Receiver<Option<Result<(SendStream, RecvStream), String>>>,
    outgoing_send_streams: flume::Receiver<Result<SendStream, String>>,
    _cb: Closure<dyn FnMut(JsValue)>,
}

impl Connection {
    /// Open a connection to `url`
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        let worker = web_sys::Worker::new("/src/networker.ts").unwrap();

        let (incoming_datagrams_tx, incoming_datagrams) = flume::unbounded();
        let (incoming_recv_streams_tx, incoming_recv_streams) = flume::unbounded();
        let (incoming_bi_streams_tx, incoming_bi_streams) = flume::unbounded();
        let (outgoing_send_streams_tx, outgoing_send_streams) = flume::unbounded();

        let (ready_tx, ready_rx) = futures::channel::oneshot::channel();
        let mut ready_tx = Some(ready_tx);

        let cb = Closure::new(move |event| {
            tracing::warn!("Worker message: {event:?}");
            let Ok(event_data) = Reflect::get(&event, &JsValue::from_str("data")) else {
                tracing::error!("Failed to get event data");
                return;
            };
            let resp = match WorkerResponse::try_from(event_data) {
                Ok(resp) => resp,
                Err(err) => {
                    tracing::error!("Failed to parse worker response: {err:?}");
                    return;
                }
            };
            match resp {
                WorkerResponse::Ready => {
                    if let Some(ready_tx) = ready_tx.take() {
                        ready_tx.send(anyhow::Ok(())).unwrap()
                    } else {
                        tracing::error!("Received multiple ready messages");
                    }
                }
                WorkerResponse::ConnectError(err) => {
                    if let Some(ready_tx) = ready_tx.take() {
                        ready_tx
                            .send(Err(anyhow::anyhow!("Connection error: {}", err)))
                            .unwrap()
                    } else {
                        tracing::error!("Received multiple ready messages");
                    }
                }
                WorkerResponse::Datagram(data) => incoming_datagrams_tx.send(data).unwrap(),
                WorkerResponse::OpenedUni(stream) => outgoing_send_streams_tx.send(stream).unwrap(),
                WorkerResponse::AcceptedUni(stream) => {
                    incoming_recv_streams_tx.send(stream).unwrap()
                }
                WorkerResponse::AcceptedBi(streams) => {
                    incoming_bi_streams_tx.send(streams).unwrap()
                }
            }
        });
        worker.set_onmessage(Some(cb.as_ref().unchecked_ref()));

        let conn = Connection {
            worker,
            incoming_datagrams,
            incoming_recv_streams,
            incoming_bi_streams,
            outgoing_send_streams,
            _cb: cb,
        };
        conn.request(WorkerRequest::Connect(url));
        ready_rx.await??;
        Ok(conn)
    }

    fn request(&self, req: WorkerRequest) {
        self.worker.post_message(&req.into()).unwrap();
    }

    pub async fn open_uni(&self) -> Result<SendStream, NetworkError> {
        self.request(WorkerRequest::OpenUni);
        self.outgoing_send_streams
            .recv_async()
            .await
            .map_err(|_| NetworkError::ConnectionClosed)?
            .map_err(|_| NetworkError::ConnectionClosed)
    }

    pub async fn open_bi(&self) -> Result<(SendStream, RecvStream), NetworkError> {
        todo!()
    }

    /// Accepts an incoming bidirectional stream
    pub async fn accept_bi(&self) -> Option<Result<(SendStream, RecvStream), NetworkError>> {
        self.request(WorkerRequest::AcceptBi);
        // FIXME: properly handle errors
        let streams = self.incoming_bi_streams.recv_async().await.transpose()?;
        let Ok(Ok(streams)) = streams else {
            return Some(Err(NetworkError::ConnectionClosed));
        };
        Some(Ok(streams))
    }

    /// Accepts an incoming unidirectional stream
    pub async fn accept_uni(&self) -> Option<Result<RecvStream, NetworkError>> {
        self.request(WorkerRequest::AcceptUni);
        // FIXME: properly handle errors
        let stream = self.incoming_recv_streams.recv_async().await.transpose()?;
        let Ok(Ok(stream)) = stream else {
            return Some(Err(NetworkError::ConnectionClosed));
        };
        Some(Ok(stream))
    }

    /// Reads the next datagram from the connection
    pub async fn read_datagram(&self) -> Option<Result<Bytes, NetworkError>> {
        self.request(WorkerRequest::PollDatagrams);
        self.incoming_datagrams
            .recv_async()
            .await
            .map_err(|_| NetworkError::ConnectionClosed)
            .transpose()
    }

    /// Sends data to a WebTransport connection.
    pub fn send_datagram(&self, data: &[u8]) {
        self.request(WorkerRequest::SendDatagram(data));
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
