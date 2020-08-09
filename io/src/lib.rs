use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::ops;

use derive_more::*;

mod triad;
pub use triad::Triad;

/// An error occurred when decoding.
pub enum DecodeError {
    UnexpectedEof,
    OutOfRange,
    InvalidUtf8,
    MagicMismatch,
}

type Result<T = (), E = DecodeError> = std::result::Result<T, E>;

/// Allows the type to be encoded/decoded using RakNet binary format.
pub trait CanIo: Sized {
    fn write(&self, vec: &mut Vec<u8>);

    fn read(src: &[u8], offset: &mut usize) -> Result<Self>;
}

/// Binary representation of a bool.
impl CanIo for bool {
    fn write(&self, vec: &mut Vec<u8>) {
        vec.push(if *self { 1 } else { 0 });
    }

    fn read(src: &[u8], offset: &mut usize) -> Result<Self> {
        match src.get(postinc(offset, 1)) {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            Some(_) => Err(DecodeError::OutOfRange),
            _ => Err(DecodeError::UnexpectedEof),
        }
    }
}

macro_rules! impl_primitive {
    ($ty:ty, $size:literal) => {
        /// Binary representation in big-endian.
        ///
        /// Wrap the type with `Little` to encode in little-endian.
        impl CanIo for $ty {
            fn write(&self, vec: &mut Vec<u8>) {
                vec.extend_from_slice(&self.to_be_bytes());
            }

            fn read(src: &[u8], offset: &mut usize) -> Result<Self> {
                let range = range_inc(offset, $size);
                match src.get(range) {
                    Some(slice) => {
                        let mut dest = [0u8; $size];
                        dest.copy_from_slice(slice);
                        Ok(<$ty>::from_be_bytes(dest))
                    }
                    None => Err(DecodeError::UnexpectedEof),
                }
            }
        }

        /// Binary representation in little-endian.
        impl CanIo for Little<$ty> {
            fn write(&self, vec: &mut Vec<u8>) {
                vec.extend_from_slice(&self.0.to_le_bytes());
            }

            fn read(src: &[u8], offset: &mut usize) -> Result<Self> {
                let range = range_inc(offset, $size);
                match src.get(range) {
                    Some(slice) => {
                        let mut dest = [0u8; $size];
                        dest.copy_from_slice(slice);
                        Ok(Self(<$ty>::from_le_bytes(dest)))
                    }
                    None => Err(DecodeError::UnexpectedEof),
                }
            }
        }
    };
}

impl_primitive!(u8, 1);
impl_primitive!(u16, 2);
impl_primitive!(u32, 4);
impl_primitive!(u64, 8);
impl_primitive!(i8, 1);
impl_primitive!(i16, 2);
impl_primitive!(i32, 4);
impl_primitive!(i64, 8);
impl_primitive!(f32, 4);
impl_primitive!(f64, 8);

/// Encodes a string using a u16 prefix indicating the length, followed by the characters encoded
/// in UTF-8.
impl CanIo for String {
    fn write(&self, vec: &mut Vec<u8>) {
        (self.len() as u16).write(&mut *vec);
        vec.extend_from_slice(self.as_bytes());
    }

    fn read(src: &[u8], offset: &mut usize) -> Result<Self> {
        let len = u16::read(src, &mut *offset)?;

        let range = range_inc(offset, len as usize);
        match src.get(range) {
            Some(slice) => String::from_utf8(slice.iter().copied().collect())
                .map_err(|_| DecodeError::InvalidUtf8),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

/// Encodes an IP address + port using RakNet format. This is a mix of standard and non-standard
/// IP encoding.
impl CanIo for SocketAddr {
    fn write(&self, vec: &mut Vec<u8>) {
        match self {
            SocketAddr::V4(addr) => {
                vec.push(4u8);
                for &byte in &addr.ip().octets() {
                    vec.push(!byte);
                }
                addr.port().write(&mut *vec);
            }
            SocketAddr::V6(addr) => {
                vec.push(6u8);
                Little(10u16).write(&mut *vec); // this should be AF_INET6, but it is platform-specific and doens't matter anyway
                addr.port().write(&mut *vec);
                addr.flowinfo().write(&mut *vec);
                vec.extend_from_slice(&addr.ip().octets());
                addr.scope_id().write(&mut *vec);
            }
        }
    }

    fn read(src: &[u8], offset: &mut usize) -> Result<Self> {
        let version = u8::read(src, &mut *offset)?;
        let ret = match version {
            4 => {
                let bytes = match src.get(range_inc(offset, 4)) {
                    Some(slice) => {
                        let mut bytes = [0u8; 4];
                        bytes.copy_from_slice(slice);
                        bytes
                    }
                    None => return Err(DecodeError::UnexpectedEof),
                };
                let port = u16::read(src, &mut *offset)?;
                SocketAddr::V4(SocketAddrV4::new(bytes.into(), port))
            }
            6 => {
                Little::<u16>::read(src, &mut *offset)?; // AF_INET6
                let port = u16::read(src, &mut *offset)?;
                let flow_info = u32::read(src, &mut *offset)?;
                let bytes = match src.get(range_inc(offset, 16)) {
                    Some(slice) => {
                        let mut bytes = [0u8; 16];
                        bytes.copy_from_slice(slice);
                        bytes
                    }
                    None => return Err(DecodeError::UnexpectedEof),
                };
                let scope_id = u32::read(src, &mut *offset)?;
                SocketAddr::V6(SocketAddrV6::new(bytes.into(), port, flow_info, scope_id))
            }
            _ => return Err(DecodeError::OutOfRange),
        };
        Ok(ret)
    }
}

fn postinc<T>(lvalue: &mut T, rvalue: T) -> T
where
    T: ops::AddAssign<T> + Clone,
{
    let clone = lvalue.clone();
    *lvalue += rvalue;
    clone
}

fn range_inc<T>(lvalue: &mut T, rvalue: T) -> ops::Range<T>
where
    T: ops::AddAssign<T> + Clone,
{
    let from = lvalue.clone();
    *lvalue += rvalue;
    let to = lvalue.clone();
    from..to
}

/// A wrapper of the primitive types, encoded in little-endian instead of big-endian.
#[derive(Clone, Copy, Debug, Default, From, PartialEq, Eq, PartialOrd, Ord)]
pub struct Little<T: Copy + Default>(pub T);

impl<T: Copy + Default> Little<T> {
    #[inline]
    pub fn inner(self) -> T {
        self.0
    }
}
