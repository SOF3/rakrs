use crate::Magic;

#[derive(Clone, Debug, rakrs_codegen::Packet)]
pub struct IncompatibleProtocolVersion {
    pub protocol_version: u8,
    pub magic: Magic,
    pub server_id: u64,
}
