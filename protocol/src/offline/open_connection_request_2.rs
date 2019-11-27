use std::net::SocketAddr;

use crate::Magic;

#[derive(Clone, Debug, rakrs_codegen::Packet, PartialEq)]
pub struct OpenConnectionRequest2 {
    pub magic: Magic,
    pub server_address: SocketAddr,
    pub mtu_size: u16,
    pub client_id: u64,
}
