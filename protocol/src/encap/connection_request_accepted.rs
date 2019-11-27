use std::net::SocketAddr;

#[derive(Clone, Debug, rakrs_codegen::Packet, PartialEq)]
pub struct ConnectionRequestAccepted {
    pub address: SocketAddr,
}
