macro_rules! packets {
    (
        $($mod:ident $name:ident $id:literal;)*
    ) => {
        $(
            mod $mod;
            pub use $mod::$name;
        )*

        /// An `EncapPacket` is a high-level packet wrapped by an `online::InnerPacket` streamed in an
        /// `online::Datagram`.
        #[derive(rakrs_codegen::Packet, PartialEq)]
        #[repr(u8)]
        pub enum EncapPacket {
            $($name($mod::$name) = $id),*
        }
    };
}

packets! [
    connected_ping ConnectedPing 0x00;
    connected_pong ConnectedPong 0x03;
    connection_request ConnectionRequest 0x09;
    connection_request_accepted ConnectionRequestAccepted 0x10;
    disconnection_notification DisconnectionNotification 0x15;
    new_incoming_connection NewIncomingConnection 0x13;
];
