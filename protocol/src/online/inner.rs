use std::io::{Error, ErrorKind, Read, Result, Write};

use rakrs_io::{CanIo, Little, Triad};

const BYTE_SIZE: u8 = 8;

const RELIABILITY_SHIFT: u8 = 5;
const RELIABILITY_MASK: u8 = (1u8 << (BYTE_SIZE - RELIABILITY_SHIFT)) - 1;
const SPLIT_BIT: u8 = 16;

#[derive(Clone, Debug, PartialEq)]
pub struct InnerPacket {
    pub reliability: InnerPacketReliability,
    pub split: Option<Split>,
    pub buffer: Vec<u8>,
}

impl InnerPacket {
    pub fn size(&self) -> usize {
        let mut size: usize = 3; // u8 + u16

        if self.reliability.reliable().is_some() {
            size += 3;
        }
        if self.reliability.sequenced().is_some() {
            size += 3;
        }
        if self.reliability.sequenced_or_ordered().is_some() {
            size += 3 + 1;
        }

        if self.split.is_some() {
            size += 4 + 2 + 4;
        }

        size += self.buffer.len();

        size
    }
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
            3 => {
                InnerPacketReliability::ReliableOrdered(CanIo::read(&mut r)?, CanIo::read(&mut r)?)
            }
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
#[derive(Clone, Debug, PartialEq)]
pub enum InnerPacketReliability {
    Unreliable,
    UnreliableSequenced(Sequenced),
    Reliable(Reliable),
    ReliableOrdered(Reliable, Ordered),
    ReliableSequenced(Reliable, Sequenced),
    UnreliableWithAckReceipt,
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
    pub fn reliable_mut(&mut self) -> Option<&mut Reliable> {
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

#[derive(Clone, Debug, rakrs_codegen::Packet, PartialEq)]
pub struct Reliable {
    pub message_index: Little<Triad>,
}

#[derive(Clone, Debug, rakrs_codegen::Packet, PartialEq)]
pub struct Sequenced {
    pub sequence_index: Little<Triad>,
    pub ordered: Ordered,
}

#[derive(Clone, Debug, rakrs_codegen::Packet, PartialEq)]
pub struct Ordered {
    pub order_index: Little<Triad>,
    pub order_channel: u8,
}

#[derive(Clone, Debug, rakrs_codegen::Packet, PartialEq)]
pub struct Split {
    pub split_count: u32,
    pub split_id: u16,
    pub split_index: u32,
}
