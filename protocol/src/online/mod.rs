use std::io::{Read, Result, Write};

use bitflags::bitflags;
use rakrs_io::CanIo;

mod ack;
mod datagram;
pub mod inner;

pub use ack::{Ack, Nack};
pub use datagram::Datagram;

bitflags! {
    #[doc = "Bitmask flags used in the leading byte of a packet of an established session."]
    pub struct Flags: u8 {
        /// Packets that do not contain this flag may be ignored.
        ///
        /// This flag should be present in all online packets.
        const VALID = 0x80;

        /// Packet ID for `Ack`.
        const ACK = 0x40;

        /// Packet ID for `Nack`.
        const NAK = 0x20;

        /// Unused flag. Should be ignored.
        #[allow(unused)]
        const PACKET_PAIR = 0x10;

        /// Unused flag. Should be ignored.
        #[allow(unused)]
        const CONTINUOUS_SEND = 0x08;

        /// Unused flag. Should be ignored.
        #[allow(unused)]
        const NEED_B_AND_AS= 0x04;
    }
}

/// Supported packets sent and received in an established session.
#[derive(Clone, Debug, PartialEq)]
pub enum OnlinePacket {
    Ack(Ack),
    Nack(Nack),
    Datagram(Datagram),
}

impl OnlinePacket {
    /// Encodes an OnlinePacket into a full UDP packet.
    pub fn write<W: Write>(&self, mut w: W) -> Result<()> {
        match self {
            OnlinePacket::Ack(ack) => {
                let flags = Flags::VALID | Flags::ACK;
                flags.bits().write(&mut w)?;
                ack.write(&mut w)
            }
            OnlinePacket::Nack(nack) => {
                let flags = Flags::VALID | Flags::NAK;
                flags.bits().write(&mut w)?;
                nack.write(&mut w)
            }
            OnlinePacket::Datagram(datagram) => {
                let flags = Flags::VALID;
                flags.bits().write(&mut w)?;
                datagram.write(&mut w)
            }
        }
    }

    /// Reads a UDP packet of unknown type and attempts to interpret it as an
    /// `OnlinePacket`.
    pub fn read<R: Read>(mut r: R) -> Result<Option<Self>> {
        let flags = Flags::from_bits_truncate(u8::read(&mut r)?);
        if !flags.contains(Flags::VALID) {
            return Ok(None);
        }

        let ret = if flags.contains(Flags::ACK) {
            OnlinePacket::Ack(CanIo::read(&mut r)?)
        } else if flags.contains(Flags::NAK) {
            OnlinePacket::Nack(CanIo::read(&mut r)?)
        } else {
            OnlinePacket::Datagram(CanIo::read(&mut r)?)
        };
        Ok(Some(ret))
    }
}
