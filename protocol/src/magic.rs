use rakrs_io::{CanIo, DecodeError};

const MAGIC_PAYLOAD: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

#[derive(Debug, Clone)]
pub struct Magic;

impl CanIo for Magic {
    fn write(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&MAGIC_PAYLOAD);
    }

    fn read(src: &[u8], offset: &mut usize) -> Result<Self, DecodeError> {
        let range = (*offset)..(*offset + 16);
        if src.get(range) == Some(&MAGIC_PAYLOAD) {
            Ok(Self)
        } else {
            Err(DecodeError::MagicMismatch)
        }
    }
}
