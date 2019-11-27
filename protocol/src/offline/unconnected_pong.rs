use crate::Magic;

#[derive(Clone, Debug, rakrs_codegen::Packet, PartialEq)]
pub struct UnconnectedPong {
    pub send_ping_time: u64,
    pub server_id: u64,
    pub magic: Magic,
    pub server_name: String,
}
