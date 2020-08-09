use std::net::SocketAddr;

use rakrs_io::{CanIo, DecodeError};

can_io! {
    enum Online : u8 {
        ConnectedPing = 0x00,
        ConnectedPong = 0x03,
        ConnectionRequest = 0x09,
        ConnectionRequestAccepted = 0x10,
        DisconnectionNotification = 0x15,
        NewIncomingConnection = 0x13,
    }
}

can_io! {
    struct ConnectedPing {
        pub send_ping_time: u64,
    }
}

can_io! {
    struct ConnectedPong {
        pub send_ping_time: u64,
        pub send_pong_time: u64,
    }
}

can_io! {
    struct ConnectionRequest {
        pub client_id: u64,
        pub send_ping_time: u64,
        pub use_security: bool,
    }
}

can_io! {
    struct ConnectionRequestAccepted {
        pub address: SocketAddr,
    }
}

can_io! {
    struct DisconnectionNotification {}
}

#[derive(Debug, Clone)]
pub struct NewIncomingConnection {
    pub address: SocketAddr,
    pub system_addresses: Vec<SocketAddr>,
    pub send_ping_time: u64,
    pub send_pong_time: u64,
}

impl CanIo for NewIncomingConnection {
    fn write(&self, vec: &mut Vec<u8>) {
        self.address.write(&mut *vec);
        for addr in &self.system_addresses {
            addr.write(&mut *vec);
        }
        self.send_ping_time.write(&mut *vec);
        self.send_pong_time.write(&mut *vec);
    }

    fn read(src: &[u8], offset: &mut usize) -> Result<Self, DecodeError> {
        let address = SocketAddr::read(src, &mut *offset)?;
        let start = *offset;
        let end = src.len() - 16;
        *offset = end;
        let send_ping_time = u64::read(src, &mut *offset)?;
        let send_pong_time = u64::read(src, &mut *offset)?;

        *offset = start;
        let mut system_addresses = vec![];
        while *offset < end {
            system_addresses.push(SocketAddr::read(src, &mut *offset)?);
        }

        if *offset != end {
            return Err(DecodeError::UnexpectedEof);
        }

        *offset = src.len();
        Ok(Self {
            address,
            system_addresses,
            send_ping_time,
            send_pong_time,
        })
    }
}
