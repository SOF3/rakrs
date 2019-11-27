use std::net::SocketAddr;

#[derive(Clone, Debug, rakrs_codegen::Packet)]
pub struct ConnectionRequestAccepted {
    pub address: SocketAddr,
}
