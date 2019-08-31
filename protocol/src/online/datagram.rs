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

use std::io::{Cursor, Read, Result, Write};

use inner::InnerPacket;
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

pub mod inner {
    #[allow(unused_imports)]
    use crate::prelude::*;

    use std::io::{Error, ErrorKind, Read, Result, Write};

    use super::{CanIo, Little, Triad};

    const BYTE_SIZE: u8 = 8;

    const RELIABILITY_SHIFT: u8 = 5;
    const RELIABILITY_MASK: u8 = (1u8 << (BYTE_SIZE - RELIABILITY_SHIFT)) - 1;
    const SPLIT_BIT: u8 = 16;

    #[derive(Clone, Debug)]
    pub struct InnerPacket {
        pub reliability: InnerPacketReliability,
        pub split: Option<Split>,
        pub buffer: Vec<u8>,
    }

    impl CanIo for InnerPacket {
        fn write<W: Write>(&self, mut w: W) -> Result<()> {
            let mut flags = self.reliability.id() << RELIABILITY_SHIFT;
            if self.split.is_some() {
                flags |= SPLIT_BIT;
            }
            flags.write(&mut w)?;

            let payload_bits: u16 = (self.buffer.len() as u16) * (BYTE_SIZE as u16);
            payload_bits.write(&mut w)?;

            match &self.reliability {
                InnerPacketReliability::Unreliable => {}
                InnerPacketReliability::UnreliableSequenced(s) => {
                    s.write(&mut w)?;
                }
                InnerPacketReliability::Reliable(r) => {
                    r.write(&mut w)?;
                }
                InnerPacketReliability::ReliableOrdered(r, o) => {
                    r.write(&mut w)?;
                    o.write(&mut w)?;
                }
                InnerPacketReliability::ReliableSequenced(r, s) => {
                    r.write(&mut w)?;
                    s.write(&mut w)?;
                }
                InnerPacketReliability::UnreliableWithAckReceipt => {}
                InnerPacketReliability::ReliableWithAckReceipt(r) => {
                    r.write(&mut w)?;
                }
                InnerPacketReliability::ReliableOrderedWithAckReceipt(r, o) => {
                    r.write(&mut w)?;
                    o.write(&mut w)?;
                }
            }

            if let Some(split) = &self.split {
                split.write(&mut w)?;
            }

            w.write_all(&self.buffer[..])?;

            Ok(())
        }

        fn read<R: Read>(mut r: R) -> Result<Self> {
            let flags = u8::read(&mut r)?;
            let has_split = (flags & RELIABILITY_SHIFT) > 0;

            let payload_bits = u16::read(&mut r)?;
            if payload_bits == 0 {
                // we have to handle this, otherwise payload_bits - 1 will panick
                return Err(Error::new(
                    ErrorKind::Other,
                    "Inner packet payload length is zero",
                ));
            }
            let payload_bytes = (payload_bits - 1) / 8 + 1; // ceil_div(payload_bits, 8)

            let reliability = match (flags >> RELIABILITY_SHIFT) & RELIABILITY_MASK {
                0 => InnerPacketReliability::Unreliable,
                1 => InnerPacketReliability::UnreliableSequenced(CanIo::read(&mut r)?),
                2 => InnerPacketReliability::Reliable(CanIo::read(&mut r)?),
                3 => InnerPacketReliability::ReliableOrdered(
                    CanIo::read(&mut r)?,
                    CanIo::read(&mut r)?,
                ),
                4 => InnerPacketReliability::ReliableSequenced(
                    CanIo::read(&mut r)?,
                    CanIo::read(&mut r)?,
                ),
                5 => InnerPacketReliability::UnreliableWithAckReceipt,
                6 => InnerPacketReliability::ReliableWithAckReceipt(CanIo::read(&mut r)?),
                7 => InnerPacketReliability::ReliableOrderedWithAckReceipt(
                    CanIo::read(&mut r)?,
                    CanIo::read(&mut r)?,
                ),
                _ => unreachable!("Already filtered with bitmask"),
            };

            let split = if has_split {
                Some(Split::read(&mut r)?)
            } else {
                None
            };

            let mut buffer = vec![0u8; payload_bytes as usize];
            r.read_exact(&mut buffer[..])?;

            Ok(Self {
                reliability,
                split,
                buffer,
            })
        }
    }

    #[repr(u8)]
    #[derive(Clone, Debug)]
    pub enum InnerPacketReliability {
        Unreliable = 0,
        UnreliableSequenced(Sequenced),
        Reliable(Reliable),
        ReliableOrdered(Reliable, Ordered),
        ReliableSequenced(Reliable, Sequenced),
        UnreliableWithAckReceipt = 5,
        ReliableWithAckReceipt(Reliable),
        ReliableOrderedWithAckReceipt(Reliable, Ordered),
    }

    impl InnerPacketReliability {
        #[inline]
        pub fn id(&self) -> u8 {
            // I don't want to rely on unstable features for ID conversion. The explicit
            // discriminants are just for potential compiler optimization
            match self {
                Self::Unreliable => 0,
                Self::UnreliableSequenced(_) => 1,
                Self::Reliable(_) => 2,
                Self::ReliableOrdered(_, _) => 3,
                Self::ReliableSequenced(_, _) => 4,
                Self::UnreliableWithAckReceipt => 5,
                Self::ReliableWithAckReceipt(_) => 6,
                Self::ReliableOrderedWithAckReceipt(_, _) => 7,
            }
        }

        #[inline]
        pub fn reliable(&self) -> Option<&Reliable> {
            match self {
                Self::Reliable(v) => Some(v),
                Self::ReliableOrdered(v, _) => Some(v),
                Self::ReliableSequenced(v, _) => Some(v),
                Self::ReliableWithAckReceipt(v) => Some(v),
                Self::ReliableOrderedWithAckReceipt(v, _) => Some(v),
                _ => None,
            }
        }

        #[inline]
        pub fn sequenced(&self) -> Option<&Sequenced> {
            match self {
                Self::UnreliableSequenced(v) => Some(v),
                Self::ReliableSequenced(_, v) => Some(v),
                _ => None,
            }
        }

        #[inline]
        pub fn ordered(&self) -> Option<&Ordered> {
            match self {
                Self::ReliableOrdered(_, v) => Some(v),
                Self::ReliableOrderedWithAckReceipt(_, v) => Some(v),
                _ => None,
            }
        }

        #[inline]
        pub fn sequenced_or_ordered(&self) -> Option<&Ordered> {
            if let Some(v) = self.ordered() {
                Some(v)
            } else if let Some(v) = self.sequenced() {
                Some(&v.ordered)
            } else {
                None
            }
        }
    }

    #[derive(Clone, Debug, Packet)]
    pub struct Reliable {
        pub message_index: Little<Triad>,
    }

    #[derive(Clone, Debug, Packet)]
    pub struct Sequenced {
        pub sequence_index: Little<Triad>,
        pub ordered: Ordered,
    }

    #[derive(Clone, Debug, Packet)]
    pub struct Ordered {
        pub order_index: Little<Triad>,
        pub order_channel: u8,
    }

    #[derive(Clone, Debug, Packet)]
    pub struct Split {
        pub split_count: u32,
        pub split_id: u16,
        pub split_index: u32,
    }
}
