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

macro_rules! packets {
    ($($mod:ident $name:ident $id:literal;)*) => {
        $(mod $mod;)*

        $(pub use $mod::$name;)*

        /// Supported packets sent and received before sessions are established.
        #[derive(Clone, Debug, Packet)]
        #[repr(u8)]
        pub enum OfflinePacket { $($name($mod::$name) = $id),* }
    };
}

// Required methods on packets:
// fn read<R: Read>(r: R) -> Result<Self>;
// fn write<W: Write>(&self, w: W) -> Result<()>;

packets! [
    incompatible_protocol_version IncompatibleProtocolVersion 0x19;
    open_connection_request_1 OpenConnectionRequest1 0x05;
    open_connection_reply_1 OpenConnectionReply1 0x06;
    open_connection_request_2 OpenConnectionRequest2 0x07;
    open_connection_reply_2 OpenConnectionReply2 0x08;
    unconnected_ping UnconnectedPing 0x01;
    unconnected_ping_open_connections UnconnectedPingOpenConnections 0x02;
    unconnected_pong UnconnectedPong 0x1c;
];
