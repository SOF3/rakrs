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
    (
        $($mod:ident $name:ident $id:literal;)*
    ) => {
        $(
            mod $mod;
            pub use $mod::$name;
        )*

        /// An `EncapPacket` is a high-level packet wrapped by an `online::InnerPacket` streamed in an
        /// `online::Datagram`.
        #[derive(Packet)]
        #[repr(u8)]
        pub enum EncapPacket {
            $($name($mod::$name) = $id),*
        }
    };
}

packets! [
    connected_ping ConnectedPing 0x00;
    connected_pong ConnectedPong 0x03;
    connection_request ConnectionRequest 0x09;
    connection_request_accepted ConnectionRequestAccepted 0x10;
    disconnection_notification DisconnectionNotification 0x15;
    new_incoming_connection NewIncomingConnection 0x13;
];
