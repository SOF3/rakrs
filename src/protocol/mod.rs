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

pub use can_io::{CanIo, Little, Triad};
pub use magic::Magic;

mod ack;
mod can_io;
mod datagram;
mod magic;

pub mod online {
    use bitflags::bitflags;

    pub use super::ack::{Ack, Nack};
    pub use super::datagram::{inner, Datagram};

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
}

#[derive(Clone, Debug)]
pub enum OnlinePacket {
    Ack(online::Ack),
    Nack(online::Nack),
    Datagram(online::Datagram),
}

impl OnlinePacket {
    pub fn write<W: Write>(&self, mut w: W) -> Result<()> {
        match self {
            OnlinePacket::Ack(ack) => {
                let flags = online::Flags::VALID | online::Flags::ACK;
                flags.bits().write(&mut w)?;
                ack.write(&mut w)
            }
            OnlinePacket::Nack(nack) => {
                let flags = online::Flags::VALID | online::Flags::NAK;
                flags.bits().write(&mut w)?;
                nack.write(&mut w)
            }
            OnlinePacket::Datagram(datagram) => {
                let flags = online::Flags::VALID;
                flags.bits().write(&mut w)?;
                datagram.write(&mut w)
            }
        }
    }

    pub fn read<R: Read>(mut r: R) -> Result<Option<Self>> {
        let flags = online::Flags::from_bits_truncate(u8::read(&mut r)?);
        if !flags.contains(online::Flags::VALID) {
            return Ok(None);
        }

        let ret = if flags.contains(online::Flags::ACK) {
            OnlinePacket::Ack(CanIo::read(&mut r)?)
        } else if flags.contains(online::Flags::NAK) {
            OnlinePacket::Nack(CanIo::read(&mut r)?)
        } else {
            OnlinePacket::Datagram(CanIo::read(&mut r)?)
        };
        Ok(Some(ret))
    }
}

macro_rules! packets {
    ($($mod:ident $name:ident $id:literal;)*) => {
        $(mod $mod;)*

        pub mod offline {
            $(pub use super::$mod::$name;)*
        }

        #[derive(Clone, Debug)]
        #[repr(u8)]
        pub enum OfflinePacket { $($name($mod::$name) = $id),* }

        impl CanIo for OfflinePacket {
            fn write<W: Write>(&self, mut w: W) -> Result<()> {
                match self {
                    $(
                        OfflinePacket::$name(var) => {
                            w.write_all(&[$id])?;
                            var.write(w)
                        },
                    )*
                }
            }

            fn read<R: Read>(mut r: R) -> Result<OfflinePacket> {
                let mut id = [0u8];
                r.read_exact(&mut id)?;
                match id[0] {
                    $(
                        $id => Ok(OfflinePacket::$name($mod::$name::read(r)?)),
                    )*
                    _ => Err(Error::new(ErrorKind::Other, "Received unknown offline packet")),
                }
            }
        }
    };
}

// Required methods on packets:
// fn read<R: Read>(r: R) -> Result<Self>;
// fn write<W: Write>(&self, w: W) -> Result<()>;

packets! [
    advertise_system AdvertiseSystem 0x1d;
    connected_ping ConnectedPing 0x00;
    connected_pong ConnectedPong 0x03;
    connection_request ConnectionRequest 0x09;
    connection_request_accepted ConnectionRequestAccepted 0x10;
    disconnection_notification DisconnectionNotification 0x15;
    incompatible_protocol_version IncompatibleProtocolVersion 0x19;
    new_incoming_connection NewIncomingConnection 0x13;
    open_connection_request_1 OpenConnectionRequest1 0x05;
    open_connection_reply_1 OpenConnectionReply1 0x06;
    open_connection_request_2 OpenConnectionRequest2 0x07;
    open_connection_reply_2 OpenConnectionReply2 0x08;
    unconnected_ping UnconnectedPing 0x01;
    unconnected_ping_open_connections UnconnectedPingOpenConnections 0x02;
    unconnected_pong UnconnectedPong 0x1c;
];
