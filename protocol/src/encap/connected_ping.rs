#[derive(Clone, Debug, rakrs_codegen::Packet, PartialEq)]
pub struct ConnectedPing {
    pub send_ping_time: u64,
}
