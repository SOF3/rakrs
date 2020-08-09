use derive_more::*;

use crate::{range_inc, CanIo, DecodeError, Little, Result};

/// A wrapper over `u32`, with only the three least-significant bytes encoded.
#[derive(Clone, Copy, Debug, Default, From, Into, PartialEq, Eq, PartialOrd, Ord)]
pub struct Triad(pub u32); // TODO check overflow

impl CanIo for Triad {
    fn write(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.0.to_be_bytes()[1..]);
    }

    fn read(src: &[u8], offset: &mut usize) -> Result<Self> {
        let range = range_inc(offset, 3);
        match src.get(range) {
            Some(slice) => {
                let mut dest = [0u8; 4]; // byte 0 MUST be 0u8
                dest[1..].copy_from_slice(slice);
                Ok(Self(u32::from_be_bytes(dest)))
            }
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl CanIo for Little<Triad> {
    fn write(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.0 .0.to_le_bytes()[..3]);
    }

    fn read(src: &[u8], offset: &mut usize) -> Result<Self> {
        let range = range_inc(offset, 3);
        match src.get(range) {
            Some(slice) => {
                let mut dest = [0u8; 4]; // byte 3 MUST be 0u8
                dest[..3].copy_from_slice(slice);
                Ok(Self(Triad(u32::from_le_bytes(dest))))
            }
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
