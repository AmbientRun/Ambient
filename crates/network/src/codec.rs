use std::marker::PhantomData;

use bytes::{Buf, BufMut};
use serde::Serialize;
use tokio_util::codec::Encoder;

pub enum ControlFrame {}

pub struct FramedCodec<T> {
    len: Option<usize>,
    _marker: PhantomData<T>,
}

impl<T> FramedCodec<T> {
    pub fn new() -> Self {
        Self {
            len: None,
            _marker: PhantomData,
        }
    }
}

impl<T: serde::de::DeserializeOwned> tokio_util::codec::Decoder for FramedCodec<T> {
    type Item = T;

    type Error = bincode::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let len = match self.len {
            Some(len) => len,
            None if src.len() >= 4 => {
                let len = src.get_u32().try_into().unwrap();
                self.len = Some(len);
                len
            }
            None => {
                src.reserve(4 - src.len());
                return Ok(None);
            }
        };

        if src.len() < len {
            return Ok(None);
        }

        let bytes = src.split_to(len);

        let value = bincode::deserialize(&bytes)?;

        // Reset state
        self.len = None;

        Ok(Some(value))
    }
}

impl<T: Serialize> Encoder<T> for FramedCodec<T> {
    type Error = bincode::Error;

    fn encode(&mut self, item: T, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        let len = bincode::serialized_size(&item)?;
        dst.reserve(len as usize + 4);

        dst.put_u32(len as u32);

        bincode::serialize_into(dst.writer(), &item)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use futures::{FutureExt, SinkExt, StreamExt};
    use tokio_util::codec::{FramedRead, FramedWrite};

    use super::*;

    #[test]
    fn framed_codec() {
        #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        enum Frame {
            String(String),
            Int(i32),
        }

        let (send, recv) = tokio::io::duplex(1024);

        let mut send = FramedWrite::new(send, FramedCodec::<Frame>::new());
        let mut recv = FramedRead::new(recv, FramedCodec::<Frame>::new());

        let mut recv = || recv.next().now_or_never().unwrap().unwrap().unwrap();

        let mut send = |frame| send.send(frame).now_or_never().unwrap().unwrap();

        send(Frame::String("Hello, World!".to_string()));

        assert_eq!(recv(), Frame::String("Hello, World!".to_string()));

        send(Frame::String("Another".to_string()));

        send(Frame::Int(42));

        assert_eq!(recv(), Frame::String("Another".into()));
        assert_eq!(recv(), Frame::Int(42));
    }
}
