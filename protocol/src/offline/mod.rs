macro_rules! packets {
    ($($mod:ident $name:ident $id:literal;)*) => {
        $(mod $mod;)*

        $(pub use $mod::$name;)*

        /// Supported packets sent and received before sessions are established.
        #[derive(Clone, Debug, rakrs_codegen::Packet, PartialEq)]
        #[repr(u8)]
        pub enum OfflinePacket { $($name($mod::$name) = $id),* }
    };
}

// Required methods on packets:
// fn read<R: Read>(r: R) -> Result<Self>;
// fn write<W: Write>(&self, w: W) -> Result<()>;

packets! [
    incompatible_protocol_version IncompatibleProtocolVersion 0x19;
    open_connection_request_1 OpenConnectionRequest1 0x05;
    open_connection_reply_1 OpenConnectionReply1 0x06;
    open_connection_request_2 OpenConnectionRequest2 0x07;
    open_connection_reply_2 OpenConnectionReply2 0x08;
    unconnected_ping UnconnectedPing 0x01;
    unconnected_ping_open_connections UnconnectedPingOpenConnections 0x02;
    unconnected_pong UnconnectedPong 0x1c;
];
