use std::io::{Cursor, Read, Result, Write};

use super::inner::InnerPacket;
use rakrs_io::{CanIo, Little, Triad};

#[derive(Clone, Debug)]
pub struct Datagram {
    pub packets: Vec<InnerPacket>,
    pub seq_number: Triad,
}

impl CanIo for Datagram {
    fn write<W: Write>(&self, mut w: W) -> Result<()> {
        Little(self.seq_number).write(&mut w)?;
        for packet in &self.packets {
            packet.write(&mut w)?;
        }
        Ok(())
    }

    fn read<R: Read>(mut r: R) -> Result<Self> {
        let seq_number = Little::<Triad>::read(&mut r)?.inner();

        let mut buf = vec![];
        r.read_to_end(&mut buf)?;
        drop(r);

        let mut packets = vec![];
        let mut cursor = Cursor::new(&buf);
        while (cursor.position() as usize) < buf.len() {
            packets.push(InnerPacket::read(&mut cursor)?);
        }

        Ok(Self {
            packets,
            seq_number,
        })
    }
}
