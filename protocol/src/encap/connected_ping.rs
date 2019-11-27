#[derive(Clone, Debug, rakrs_codegen::Packet)]
pub struct ConnectedPing {
    pub send_ping_time: u64,
}
