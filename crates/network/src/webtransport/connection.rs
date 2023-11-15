use std::{
    collections::HashMap,
    io,
    pin::Pin,
    sync::{Arc, Weak},
    task::{Context, Poll},
};

use bytes::{Buf, Bytes};
use futures::StreamExt;
use js_sys::{Number, Reflect, Uint8Array};
use parking_lot::Mutex;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::Worker;

use crate::NetworkError;

enum WorkerRequest<'a> {
    Connect(&'a str),
    PollDatagrams,
    SendDatagram(&'a [u8]),
    OpenUni,
    AcceptUni,
    AcceptBi,
    SendStreamData(u32, &'a [u8]),
    PollStream(u32),
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
            Self::SendStreamData(stream_id, data) => {
                let array = js_sys::Array::new_with_length(3);
                array.set(0, JsValue::from("send_stream_data")); // TODO: turn to ints
                array.set(1, Number::from(stream_id).into());
                array.set(2, Uint8Array::from(data).into());
                array.into()
            }
            Self::PollStream(stream_id) => {
                let array = js_sys::Array::new_with_length(2);
                array.set(0, JsValue::from("poll_stream")); // TODO: turn to ints
                array.set(1, Number::from(stream_id).into());
                array.into()
            }
        }
    }
}

enum WorkerResponse {
    Ready,
    ConnectError(String),
    Datagram(Option<Bytes>),
    OpenedUni(Result<u32, String>),
    AcceptedUni(Option<Result<u32, String>>),
    AcceptedBi(Option<Result<u32, String>>),
    ReceivedStreamData(u32, Bytes),
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
                let val = array.get(1);
                let result = val
                    .as_f64()
                    .map(|id| id as u32)
                    .ok_or_else(|| format!("{:?}", val));
                Ok(Self::OpenedUni(result))
            }
            "accepted_uni" => {
                let val = array.get(1);
                if val.is_null() {
                    Ok(Self::AcceptedUni(None))
                } else {
                    let result = val
                        .as_f64()
                        .map(|id| id as u32)
                        .ok_or_else(|| format!("{:?}", val));
                    Ok(Self::AcceptedUni(Some(result)))
                }
            }
            "accepted_bi" => {
                let val = array.get(1);
                if val.is_null() {
                    Ok(Self::AcceptedBi(None))
                } else {
                    let result = val
                        .as_f64()
                        .map(|id| id as u32)
                        .ok_or_else(|| format!("{:?}", val));
                    Ok(Self::AcceptedBi(Some(result)))
                }
            }
            "received_stream_data" => {
                let val = array.get(1);
                let Some(stream_id) = val.as_f64().map(|id| id as u32) else {
                    return Err("missing stream id");
                };
                let data = array.get(2).dyn_into::<Uint8Array>().unwrap();
                Ok(Self::ReceivedStreamData(stream_id, data.to_vec().into()))
            }
            _ => Err("unknown response"),
        }
    }
}

/// The webtransport connection
///
/// Disconnects when dropped
pub struct Connection {
    worker: Arc<Mutex<Worker>>,
    incoming_datagrams: flume::Receiver<Option<Bytes>>,
    incoming_recv_streams: flume::Receiver<Option<Result<u32, String>>>,
    incoming_bi_streams: flume::Receiver<Option<Result<u32, String>>>,
    outgoing_send_streams: flume::Receiver<Result<u32, String>>,
    read_channels: Arc<Mutex<HashMap<u32, flume::Sender<Bytes>>>>,
    _cb: Closure<dyn FnMut(JsValue)>,
}

impl Connection {
    /// Open a connection to `url`
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        let worker = Arc::new(Mutex::new(
            web_sys::Worker::new("/src/networker.ts").unwrap(),
        ));
        let read_channels = Arc::new(Mutex::new(HashMap::<u32, flume::Sender<Bytes>>::new()));

        let (incoming_datagrams_tx, incoming_datagrams) = flume::unbounded();
        let (incoming_recv_streams_tx, incoming_recv_streams) = flume::unbounded();
        let (incoming_bi_streams_tx, incoming_bi_streams) = flume::unbounded();
        let (outgoing_send_streams_tx, outgoing_send_streams) = flume::unbounded();

        let (ready_tx, ready_rx) = futures::channel::oneshot::channel();
        let mut ready_tx = Some(ready_tx);

        let cb = Closure::new({
            let read_channels = read_channels.clone();
            move |event| {
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
                    WorkerResponse::OpenedUni(stream) => {
                        outgoing_send_streams_tx.send(stream).unwrap()
                    }
                    WorkerResponse::AcceptedUni(stream) => {
                        incoming_recv_streams_tx.send(stream).unwrap()
                    }
                    WorkerResponse::AcceptedBi(streams) => {
                        incoming_bi_streams_tx.send(streams).unwrap()
                    }
                    WorkerResponse::ReceivedStreamData(stream_id, data) => {
                        if let Some(tx) = read_channels.lock().get(&stream_id) {
                            tx.send(data).unwrap();
                        } else {
                            tracing::error!("Received data for unknown stream {}", stream_id);
                        }
                    }
                }
            }
        });
        worker
            .lock()
            .set_onmessage(Some(cb.as_ref().unchecked_ref()));

        let conn = Connection {
            worker,
            incoming_datagrams,
            incoming_recv_streams,
            incoming_bi_streams,
            outgoing_send_streams,
            read_channels,
            _cb: cb,
        };
        conn.request(WorkerRequest::Connect(url));
        ready_rx.await??;
        Ok(conn)
    }

    fn request(&self, req: WorkerRequest) {
        self.worker.lock().post_message(&req.into()).unwrap();
    }

    pub async fn open_uni(&self) -> Result<SendStream, NetworkError> {
        self.request(WorkerRequest::OpenUni);
        let stream_id = self.outgoing_send_streams.recv_async().await;
        let Ok(Ok(stream_id)) = stream_id else {
            return Err(NetworkError::ConnectionClosed);
        };
        Ok(SendStream {
            stream_id,
            worker: Arc::downgrade(&self.worker),
        })
    }

    pub async fn open_bi(&self) -> Result<(SendStream, RecvStream), NetworkError> {
        todo!()
    }

    /// Accepts an incoming bidirectional stream
    pub async fn accept_bi(&self) -> Option<Result<(SendStream, RecvStream), NetworkError>> {
        self.request(WorkerRequest::AcceptBi);
        // FIXME: properly handle errors
        let stream_id = self.incoming_bi_streams.recv_async().await.transpose()?;
        let Ok(Ok(stream_id)) = stream_id else {
            return Some(Err(NetworkError::ConnectionClosed));
        };
        let (tx, rx) = flume::unbounded();
        self.read_channels.lock().insert(stream_id, tx);
        Some(Ok((
            SendStream {
                stream_id,
                worker: Arc::downgrade(&self.worker),
            },
            RecvStream {
                stream_id,
                worker: Arc::downgrade(&self.worker),
                rx: Box::pin(rx.into_stream()),
                buf: Bytes::new(),
            },
        )))
    }

    /// Accepts an incoming unidirectional stream
    pub async fn accept_uni(&self) -> Option<Result<RecvStream, NetworkError>> {
        self.request(WorkerRequest::AcceptUni);
        // FIXME: properly handle errors
        let stream_id = self.incoming_recv_streams.recv_async().await.transpose()?;
        let Ok(Ok(stream_id)) = stream_id else {
            return Some(Err(NetworkError::ConnectionClosed));
        };
        let (tx, rx) = flume::unbounded();
        self.read_channels.lock().insert(stream_id, tx);
        Some(Ok(RecvStream {
            stream_id,
            worker: Arc::downgrade(&self.worker),
            rx: Box::pin(rx.into_stream()),
            buf: Bytes::new(),
        }))
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

pub struct SendStream {
    stream_id: u32,
    worker: Weak<Mutex<Worker>>,
}

impl AsyncWrite for SendStream {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        if let Some(worker) = self.worker.upgrade() {
            worker
                .lock()
                .post_message(&WorkerRequest::SendStreamData(self.stream_id, buf).into())
                .unwrap();
            Poll::Ready(Ok(buf.len()))
        } else {
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Connection closed",
            )))
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(
            self.worker
                .upgrade()
                .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "Connection closed"))
                .map(|_| ()),
        )
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        todo!()
    }
}

pub struct RecvStream {
    stream_id: u32,
    worker: Weak<Mutex<Worker>>,
    rx: Pin<Box<dyn futures::Stream<Item = Bytes>>>,
    buf: Bytes,
}

impl AsyncRead for RecvStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), io::Error>> {
        if let Some(worker) = self.worker.upgrade() {
            if self.buf.has_remaining() {
                let len = buf.remaining().min(self.buf.len());

                buf.put_slice(&self.buf[..len]);
                self.buf.advance(len);

                return Poll::Ready(Ok(()));
            }

            worker
                .lock()
                .post_message(&WorkerRequest::PollStream(self.stream_id).into())
                .unwrap();

            self.rx.poll_next_unpin(cx).map(|data| {
                if let Some(data) = data {
                    self.buf = data;
                    let len = buf.remaining().min(self.buf.len());
                    buf.put_slice(&self.buf[..len]);
                    self.buf.advance(len);
                }
                Ok(())
            })
        } else {
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Connection closed",
            )))
        }
    }
}
