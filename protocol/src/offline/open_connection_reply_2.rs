use std::net::SocketAddr;

use crate::Magic;

#[derive(Clone, Debug, rakrs_codegen::Packet, PartialEq)]
pub struct OpenConnectionReply2 {
    pub magic: Magic,
    pub server_id: u64,
    pub client_address: SocketAddr,
    pub mtu_size: u16,
    pub server_security: bool,
}
