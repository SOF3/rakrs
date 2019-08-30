// rakrs
// Copyright (C) SOFe
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[allow(unused_imports)]
use crate::prelude::*;

use std::io::{Error, ErrorKind, Read, Result, Write};
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::ops::{Deref, DerefMut};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

pub trait CanIo: Sized {
    fn write<W: Write>(&self, w: W) -> Result<()>;

    fn read<R: Read>(r: R) -> Result<Self>;
}

#[derive(From)]
pub struct Little<T: Copy>(pub T);

impl<T: Copy> Deref for Little<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Copy> DerefMut for Little<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl CanIo for bool {
    fn write<W: Write>(&self, mut w: W) -> Result<()> {
        w.write_u8(if *self { 1 } else { 0 })
    }

    fn read<R: Read>(mut r: R) -> Result<Self> {
        match r.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::new(ErrorKind::Other, "Received invalid value for bool")),
        }
    }
}

impl CanIo for u8 {
    fn write<W: Write>(&self, mut w: W) -> Result<()> {
        w.write_u8(*self)
    }

    fn read<R: Read>(mut r: R) -> Result<Self> {
        r.read_u8()
    }
}

impl CanIo for i8 {
    fn write<W: Write>(&self, mut w: W) -> Result<()> {
        w.write_i8(*self)
    }

    fn read<R: Read>(mut r: R) -> Result<Self> {
        r.read_i8()
    }
}

macro_rules! impl_primitive {
    ($ty:ident, $write:ident, $read:ident) => {
        impl CanIo for $ty {
            fn write<W: Write>(&self, mut w: W) -> Result<()> {
                w.$write::<BigEndian>(*self)
            }

            fn read<R: Read>(mut r: R) -> Result<Self> {
                r.$read::<BigEndian>()
            }
        }

        impl CanIo for Little<$ty> {
            fn write<W: Write>(&self, mut w: W) -> Result<()> {
                w.$write::<LittleEndian>(**self)
            }

            fn read<R: Read>(mut r: R) -> Result<Self> {
                Ok(r.$read::<LittleEndian>()?.into())
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
            },
            6 => {
                Little::<u16>::read(&mut r)?; // AF_INET6
                let port = u16::read(&mut r)?;
                let flow_info = u32::read(&mut r)?;
                let mut bytes = [0u8; 16];
                r.read_exact(&mut bytes)?;
                let scope_id = u32::read(&mut r)?;
                SocketAddr::V6(SocketAddrV6::new(bytes.into(), port, flow_info, scope_id))
            },
            _ => Err(Error::new(ErrorKind::Other, "Received unsupported IP version"))?
        };
        Ok(ret)
    }
}
