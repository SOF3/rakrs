use crate::Magic;

#[derive(Clone, Debug, rakrs_codegen::Packet, PartialEq)]
pub struct UnconnectedPing {
    pub send_ping_time: u64,
    pub magic: Magic,
    pub client_id: u64,
}
