use std::io::{Error, ErrorKind, Read, Result, Write};
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

pub use little::Little;
pub use triad::Triad;

mod little;
mod triad;

/// Allows the type to be encoded/decoded using RakNet binary format.
pub trait CanIo: Sized {
    fn write<W: Write>(&self, w: W) -> Result<()>;

    fn read<R: Read>(r: R) -> Result<Self>;
}

/// Binary representation of a bool.
impl CanIo for bool {
    fn write<W: Write>(&self, mut w: W) -> Result<()> {
        w.write_u8(if *self { 1 } else { 0 })
    }

    fn read<R: Read>(mut r: R) -> Result<Self> {
        match r.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::new(
                ErrorKind::Other,
                "Received invalid value for bool",
            )),
        }
    }
}

/// Binary representation of an unsigned byte.
impl CanIo for u8 {
    fn write<W: Write>(&self, mut w: W) -> Result<()> {
        w.write_u8(*self)
    }

    fn read<R: Read>(mut r: R) -> Result<Self> {
        r.read_u8()
    }
}

/// Binary representation of a signed byte.
impl CanIo for i8 {
    fn write<W: Write>(&self, mut w: W) -> Result<()> {
        w.write_i8(*self)
    }

    fn read<R: Read>(mut r: R) -> Result<Self> {
        r.read_i8()
    }
}

macro_rules! impl_primitive {
    ($ty:ty, $write:ident, $read:ident) => {
        impl_primitive!($ty, $ty, $write, $read);
    };
    ($ty:ty, $intermediate:ty, $write:ident, $read:ident) => {
        /// Binary representation in big-endian.
        ///
        /// Wrap the type with `Little` to encode in little-endian.
        impl CanIo for $ty {
            fn write<W: Write>(&self, mut w: W) -> Result<()> {
                let value = *self;
                w.$write::<BigEndian>(value.into())
            }

            fn read<R: Read>(mut r: R) -> Result<Self> {
                let value = r.$read::<BigEndian>()?;
                Ok(value.into())
            }
        }

        /// Binary representation in little-endian.
        impl CanIo for Little<$ty> {
            fn write<W: Write>(&self, mut w: W) -> Result<()> {
                let value = *self;
                w.$write::<LittleEndian>(value.inner().into())
            }

            fn read<R: Read>(mut r: R) -> Result<Self> {
                let raw = r.$read::<LittleEndian>()?;
                let intermediate = <$intermediate>::from(raw);
                Ok(Little::from(intermediate))
            }
        }
    };
}

impl_primitive!(u16, write_u16, read_u16);
impl_primitive!(u32, write_u32, read_u32);
impl_primitive!(u64, write_u64, read_u64);
impl_primitive!(i16, write_i16, read_i16);
impl_primitive!(i32, write_i32, read_i32);
impl_primitive!(i64, write_i64, read_i64);
impl_primitive!(f32, write_f32, read_f32);
impl_primitive!(f64, write_f64, read_f64);
impl_primitive!(Triad, write_u24, read_u24);

/// Encodes a string using a u16 prefix indicating the length, followed by the characters encoded
/// in UTF-8.
impl CanIo for String {
    fn write<W: Write>(&self, mut w: W) -> Result<()> {
        CanIo::write(&(self.len() as u16), &mut w)?;
        w.write_all(self.as_bytes())
    }

    fn read<R: Read>(mut r: R) -> Result<Self> {
        let len: u16 = CanIo::read(&mut r)?;
        let mut buf = vec![0u8; len as usize];
        r.read_exact(&mut buf)?;
        match String::from_utf8(buf) {
            Ok(string) => Ok(string),
            Err(err) => Err(Error::new(ErrorKind::Other, err)),
        }
    }
}

/// Encodes an IP address + port using RakNet format. This is a mix of standard and non-standard
/// IP encoding.
impl CanIo for SocketAddr {
    fn write<W: Write>(&self, mut w: W) -> Result<()> {
        match self {
            SocketAddr::V4(addr) => {
                4u8.write(&mut w)?;
                for &byte in &addr.ip().octets() {
                    (!byte).write(&mut w)?;
                }
                addr.port().write(&mut w)?;
            }
            SocketAddr::V6(addr) => {
                6u8.write(&mut w)?;
                Little(10u16).write(&mut w)?; // this should be AF_INET6, but it is platform-specific and doens't matter anyway
                addr.port().write(&mut w)?;
                addr.flowinfo().write(&mut w)?;
                w.write_all(&addr.ip().octets())?; // TODO verify if this implements the protocol correctly
                addr.scope_id().write(&mut w)?;
            }
        }
        Ok(())
    }

    fn read<R: Read>(mut r: R) -> Result<Self> {
        let version = u8::read(&mut r)?;
        let ret = match version {
            4 => {
                let mut bytes = [0u8; 4];
                r.read_exact(&mut bytes)?;
                let port = u16::read(&mut r)?;
                SocketAddr::V4(SocketAddrV4::new(bytes.into(), port))
            }
            6 => {
                Little::<u16>::read(&mut r)?; // AF_INET6
                let port = u16::read(&mut r)?;
                let flow_info = u32::read(&mut r)?;
                let mut bytes = [0u8; 16];
                r.read_exact(&mut bytes)?;
                let scope_id = u32::read(&mut r)?;
                SocketAddr::V6(SocketAddrV6::new(bytes.into(), port, flow_info, scope_id))
            }
            _ => Err(Error::new(
                ErrorKind::Other,
                "Received unsupported IP version",
            ))?,
        };
        Ok(ret)
    }
}
