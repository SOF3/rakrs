#[derive(Clone, Debug, rakrs_codegen::Packet)]
pub struct ConnectedPong {
    pub send_ping_time: u64,
    pub send_pong_time: u64,
}
