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

use std::io::{Read, Result, Write};

use bitflags::bitflags;
use rakrs_io::CanIo;

mod ack;
mod datagram;

pub use ack::{Ack, Nack};
pub use datagram::{inner, Datagram};

bitflags! {
    pub struct Flags: u8 {
        const VALID = 0x80;
        const ACK = 0x40;
        const NAK = 0x20;
        const PACKET_PAIR = 0x10;
        const CONTINUOUS_SEND = 0x08;
        const NEED_B_AND_AS= 0x04;
    }
}

#[derive(Clone, Debug)]
pub enum OnlinePacket {
    Ack(Ack),
    Nack(Nack),
    Datagram(Datagram),
}

impl OnlinePacket {
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
