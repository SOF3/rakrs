use std::io::{Error, ErrorKind, Read, Result, Write};

use rakrs_io::CanIo;

/// Handles the 16-byte magic sequence in RakNet protocol.
/// This is a marker type and does not take any memory.
#[derive(Clone, Debug, PartialEq)]
pub struct Magic;

const MAGIC_PAYLOAD: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

impl CanIo for Magic {
    /// Writes the magic sequence to the stream.
    fn write<W: Write>(&self, mut w: W) -> Result<()> {
        w.write_all(&MAGIC_PAYLOAD)
    }

    /// Reads the magic sequence from the stream and validates it.
    fn read<R: Read>(mut r: R) -> Result<Self> {
        let mut payload = [0u8; 16];
        r.read_exact(&mut payload)?;
        if &payload == &MAGIC_PAYLOAD {
            Ok(Self)
        } else {
            Err(Error::new(ErrorKind::Other, "Magic payload mismatch"))
        }
    }
}

#[cfg(test)]
rakrs_testkit::canio_ok! {
    test_write:
        0x00, 0xff, 0xff, 0x00,
        0xfe, 0xfe, 0xfe, 0xfe,
        0xfd, 0xfd, 0xfd, 0xfd,
        0x12, 0x34, 0x56, 0x78,
    = test_read: Magic
}

#[cfg(test)]
rakrs_testkit::canio_err_read! {
    test_bad_read_1: Magic => "Magic payload mismatch";
        0x01, 0xff, 0xff, 0x00,
        0xfe, 0xfe, 0xfe, 0xfe,
        0xfd, 0xfd, 0xfd, 0xfd,
        0x12, 0x34, 0x56, 0x78,
}

#[cfg(test)]
rakrs_testkit::canio_err_read! {
    test_bad_read_2: Magic => "Magic payload mismatch";
        0x01, 0xff, 0xff, 0x00,
        0xfe, 0xfe, 0xfe, 0xfe,
        0xfd, 0xfd, 0xfd, 0xfd,
        0x12, 0x34, 0x56, 0x70,
}
