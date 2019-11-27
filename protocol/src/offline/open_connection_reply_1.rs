use crate::Magic;

#[derive(Clone, Debug, rakrs_codegen::Packet, PartialEq)]
pub struct OpenConnectionReply1 {
    pub magic: Magic,
    pub server_id: u64,
    pub server_security: bool,
    pub mtu_size: u16,
}
