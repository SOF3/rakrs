use std::net::SocketAddr;

use rakrs_io::{CanIo, DecodeError};

use crate::Magic;

can_io! {
    enum Offline: u8 {
        IncompatibleProtocolVersion = 0x19,
        OpenConnectionRequest1 = 0x05,
        OpenConnectionReply1 = 0x06,
        OpenConnectionRequest2 = 0x07,
        OpenConnectionReply2 = 0x08,
        UnconnectedPing = 0x01,
        UnconnectedPingOpenConnections = 0x02,
        UnconnectedPong = 0x1c,
    }
}

can_io! {
    struct IncompatibleProtocolVersion {
        pub protocol_version: u8,
        pub magic: Magic,
        pub server_id: u64,
    }
}

#[derive(Debug, Clone)]
pub struct OpenConnectionRequest1 {
    pub magic: Magic,
    pub protocol: u8,
    pub mtu_size: usize,
}

impl CanIo for OpenConnectionRequest1 {
    fn write(&self, vec: &mut Vec<u8>) {
        self.magic.write(&mut *vec);
        self.protocol.write(&mut *vec);
        vec.extend(std::iter::repeat(0u8).take(self.mtu_size - vec.len()));
    }

    fn read(src: &[u8], offset: &mut usize) -> Result<Self, DecodeError> {
        let magic = Magic::read(src, &mut *offset)?;
        let protocol = u8::read(src, &mut *offset)?;
        let mtu_size = src.len();
        *offset = src.len();
        Ok(Self {
            magic,
            protocol,
            mtu_size,
        })
    }
}

can_io! {
    struct OpenConnectionReply1 {
        pub magic: Magic,
        pub server_id: u64,
        pub server_security: bool,
        pub mtu_size: u16,
    }
}

can_io! {
    struct OpenConnectionRequest2 {
        pub magic: Magic,
        pub server_address: SocketAddr,
        pub mtu_size: u16,
        pub client_id: u64,
    }
}

can_io! {
    struct OpenConnectionReply2 {
        pub magic: Magic,
        pub server_id: u64,
        pub client_address: SocketAddr,
        pub mtu_size: u16,
        pub server_security: bool,
    }
}

can_io! {
    struct UnconnectedPing {
        pub send_ping_time: u64,
        pub magic: Magic,
        pub client_id: u64,
    }
}

can_io! {
    struct UnconnectedPingOpenConnections {
        pub send_ping_time: u64,
        pub magic: Magic,
        pub client_id: u64,
    }
}

can_io! {
    struct UnconnectedPong {
        pub send_ping_time: u64,
        pub server_id: u64,
        pub magic: Magic,
        pub server_name: String,
    }
}
