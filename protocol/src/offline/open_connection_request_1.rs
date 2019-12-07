use std::io::{Read, Result, Write};

use crate::Magic;
use rakrs_io::CanIo;

#[derive(Clone, Debug, PartialEq)]
pub struct OpenConnectionRequest1 {
    pub magic: Magic,
    pub protocol: u8,
    pub mtu_size: usize,
}

impl CanIo for OpenConnectionRequest1 {
    fn write<W: Write>(&self, mut w: W) -> Result<()> {
        self.magic.write(&mut w)?;
        self.protocol.write(&mut w)?;

        w.write_all(&mut vec![0u8; self.mtu_size])?;
        Ok(())
    }

    fn read<R: Read>(mut r: R) -> Result<Self> {
        let magic = <Magic as CanIo>::read(&mut r)?;
        let protocol = <u8 as CanIo>::read(&mut r)?;
        let mtu_size = r.bytes().count();

        Ok(Self {
            magic,
            protocol,
            mtu_size,
        })
    }
}
