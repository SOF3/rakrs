use bitflags::bitflags;
use rakrs_io::{CanIo, DecodeError, Little, Triad};

use crate::{inner, Inner};

bitflags! {
    /// Bitmask flags used in the leading byte of a packet of an #established session.
    pub struct Flags: u8 {
        /// Packets that do not contain this flag may be ignored.
        ///
        /// This flag should be present in all online packets.
        const VALID = 0x80;
        /// Packet ID for `Ack`.
        const ACK = 0x40;
        /// Packet ID for `Nack`.
        const NAK = 0x20;
        /// Unused flag. Should be ignored.
        #[allow(unused)]
        const PACKET_PAIR = 0x10;
        /// Unused flag. Should be ignored.
        #[allow(unused)]
        const CONTINUOUS_SEND = 0x08;
        /// Unused flag. Should be ignored.
        #[allow(unused)]
        const NEED_B_AND_AS = 0x04;
    }
}

#[derive(Debug, Clone)]
pub enum ReliableOnline<B: inner::Buffer>
where
    for<'t> &'t B: IntoIterator<Item = &'t u8>,
{
    None,
    Ack(TriadList),
    Nack(TriadList),
    Datagram(Datagram<B>),
}

impl<B: inner::Buffer> ReliableOnline<B>
where
    for<'t> &'t B: IntoIterator<Item = &'t u8>,
{
    pub fn write(&self, vec: &mut Vec<u8>) {
        match self {
            Self::None => vec.push(0),
            Self::Ack(list) => {
                let flag = Flags::VALID | Flags::ACK;
                vec.push(flag.bits());
                list.write(vec);
            }
            Self::Nack(list) => {
                let flag = Flags::VALID | Flags::NAK;
                vec.push(flag.bits());
                list.write(vec);
            }
            Self::Datagram(datagram) => {
                vec.push(Flags::VALID.bits());
                datagram.write(vec);
            }
        }
    }
}

impl ReliableOnline<Vec<u8>> {
    pub fn read(src: &[u8], offset: &mut usize) -> Result<Self, DecodeError> {
        let flag = Flags::from_bits_truncate(u8::read(src, &mut *offset)?);
        if !flag.contains(Flags::VALID) {
            return Ok(Self::None);
        }
        if flag.contains(Flags::ACK) {
            Ok(Self::Ack(TriadList::read(src, &mut *offset)?))
        } else if flag.contains(Flags::NAK) {
            Ok(Self::Nack(TriadList::read(src, &mut *offset)?))
        } else {
            Ok(Self::Datagram(Datagram::read(src, &mut *offset)?))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Datagram<B: inner::Buffer>
where
    for<'t> &'t B: IntoIterator<Item = &'t u8>,
{
    pub seq_number: u32,
    pub packets: Vec<Inner<B>>,
}

impl<B: inner::Buffer> Datagram<B>
where
    for<'t> &'t B: IntoIterator<Item = &'t u8>,
{
    pub fn write(&self, vec: &mut Vec<u8>) {
        Little(Triad(self.seq_number)).write(&mut *vec);
        for packet in &self.packets {
            packet.write(&mut *vec);
        }
    }
}

impl Datagram<Vec<u8>> {
    pub fn read(src: &[u8], offset: &mut usize) -> Result<Self, DecodeError> {
        let Little(Triad(seq_number)) = CanIo::read(src, &mut *offset)?;
        let mut packets = vec![];
        while *offset < src.len() {
            packets.push(Inner::read(src, &mut *offset)?);
        }
        Ok(Self {
            seq_number,
            packets,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TriadList {
    pairs: Vec<(u32, u32)>,
}

const RECORD_TYPE_RANGE: u8 = 0;
const RECORD_TYPE_SINGLE: u8 = 1;

impl TriadList {
    pub fn from_sorted(mut list: impl Iterator<Item = u32>) -> Self {
        let mut start = match list.next() {
            Some(id) => id,
            None => return Self { pairs: vec![] },
        };
        let mut end = start;

        let mut pairs = vec![];

        for id in list {
            debug_assert!(id <= end, "List is not strictly increasing");
            if id == end + 1 {
                end = id;
                continue;
            }
            pairs.push((start, end));
            start = id;
            end = id;
        }
        Self { pairs }
    }

    pub fn size(&self) -> u32 {
        self.pairs
            .iter()
            .map(|(start, end)| (end - start + 1))
            .sum()
    }

    pub fn to_sorted<'t>(&'t self) -> impl Iterator<Item = u32> + 't {
        self.pairs.iter().flat_map(|&(start, end)| start..=end)
    }
}

impl CanIo for TriadList {
    fn write(&self, vec: &mut Vec<u8>) {
        for &(start, end) in &self.pairs {
            if start == end {
                RECORD_TYPE_SINGLE.write(&mut *vec);
                Little(Triad(start)).write(&mut *vec);
            } else {
                RECORD_TYPE_RANGE.write(&mut *vec);
                Little(Triad(start)).write(&mut *vec);
                Little(Triad(end)).write(&mut *vec);
            }
        }
    }

    fn read(src: &[u8], offset: &mut usize) -> Result<Self, DecodeError> {
        let mut pairs = Vec::new();
        while *offset < src.len() {
            let record_type = u8::read(src, &mut *offset)?;
            let pair = match record_type {
                RECORD_TYPE_SINGLE => {
                    let Little(Triad(id)) = Little::<Triad>::read(src, &mut *offset)?;
                    (id, id)
                }
                RECORD_TYPE_RANGE => {
                    let Little(Triad(start)) = Little::<Triad>::read(src, &mut *offset)?;
                    let Little(Triad(end)) = Little::<Triad>::read(src, &mut *offset)?;
                    (start, end)
                }
                _ => return Err(DecodeError::UnexpectedEof),
            };
            pairs.push(pair);
        }

        pairs.sort_by_key(|&(start, _end)| start);

        let mut last = None;
        for (start, end) in &mut pairs {
            if let Some(last) = last {
                if *start <= last {
                    *start = last + 1; // deduplication
                }
            }
            last = Some(*end);
        }

        Ok(Self { pairs })
    }
}
