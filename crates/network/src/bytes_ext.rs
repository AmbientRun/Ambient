use bytes::Buf;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
#[error("Unexpected end when reading {0} bytes")]
pub struct UnexpectedEnd(usize);

pub trait Decode {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self, UnexpectedEnd>
    where
        Self: Sized;
}

/// Allows fallible buffer decoding
pub trait BufExt: Buf {
    fn try_get_u8(&mut self) -> Result<u8, UnexpectedEnd> {
        self.try_get()
    }

    fn try_get_u16(&mut self) -> Result<u16, UnexpectedEnd> {
        self.try_get()
    }

    fn try_get_u32(&mut self) -> Result<u32, UnexpectedEnd> {
        self.try_get()
    }

    fn try_get_u64(&mut self) -> Result<u64, UnexpectedEnd> {
        self.try_get()
    }

    fn try_get_u128(&mut self) -> Result<u64, UnexpectedEnd> {
        self.try_get()
    }

    fn try_get<T>(&mut self) -> Result<T, UnexpectedEnd>
    where
        T: Sized + Decode,
    {
        let mut this = self;
        T::decode(&mut this)
    }
}

macro_rules! impl_decode_u {
    ($ty: ty, $n: literal, $m: ident) => {
        impl Decode for $ty {
            fn decode<B: Buf>(buf: &mut B) -> Result<Self, UnexpectedEnd> {
                if buf.remaining() < $n {
                    return Err(UnexpectedEnd($n));
                }

                Ok(buf.$m())
            }
        }
    };
}

impl_decode_u!(u8, 1, get_u8);
impl_decode_u!(u16, 2, get_u16);
impl_decode_u!(u32, 4, get_u32);
impl_decode_u!(u64, 8, get_u64);

impl<B> BufExt for B where B: Buf {}
