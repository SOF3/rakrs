#[derive(Clone, Debug, rakrs_codegen::Packet)]
pub struct ConnectionRequest {
    pub client_id: u64,
    pub send_ping_time: u64,
    pub use_security: bool,
}
