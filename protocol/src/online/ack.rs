use std::io::{Error, ErrorKind, Read, Result, Write};
use std::iter::Iterator;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use super::CanIo;

const RECORD_TYPE_RANGE: u8 = 0;
const RECORD_TYPE_SINGLE: u8 = 1;

type PacketNum = u32;
#[derive(Clone, Debug, PartialEq, Eq)]
struct Cluster(PacketNum, PacketNum);

fn cluster<I>(packets: I) -> Vec<Cluster>
where
    I: Iterator<Item = PacketNum>,
{
    // TODO rewrite this with generators when it's stabilized

    let mut cluster: Option<Cluster> = None;
    let mut history = Vec::<Cluster>::new();

    for packet in packets {
        if let Some(cluster) = &mut cluster {
            if packet == cluster.1 + 1 {
                cluster.1 = packet;
                continue;
            }
            history.push(cluster.clone());
        }
        cluster = Some(Cluster(packet, packet));
    }
    if let Some(cluster) = cluster {
        history.push(cluster);
    }

    history
}

#[cfg(test)]
mod cluster_test {
    macro_rules! make_cluster_tests {
        ( $( $name:ident $($raw:literal),* | $([$start:literal, $end:literal])*;)*) => {
            $(
                mod $name {
                    use super::super::*;

                    #[test]
                    fn test_cluster() {
                        let input = [$($raw),*];
                        let mut clusters = cluster(input.iter().copied()).into_iter();

                        $(
                            assert_eq!(clusters.next(), Some(Cluster($start, $end)));
                        )*
                        assert_eq!(clusters.next(), None);
                    }
                }
            )*
        };
    }

    make_cluster_tests!(
        single 2 | [2, 2];
        continuous  2, 3, 4 | [2, 4];
        single_single 2, 4 | [2, 2] [4, 4];
        single_continuous 2, 4, 5 | [2, 2] [4, 5];
        continuous_single 2, 3, 5 | [2, 3] [5, 5];
        continuous_continuous 2, 3, 5, 6 | [2, 3] [5, 6];
    );
}

fn write_cluster<W: Write>(cluster: Cluster, mut w: W) -> Result<()> {
    if cluster.0 == cluster.1 {
        w.write_u8(RECORD_TYPE_SINGLE)?;
        w.write_u24::<LittleEndian>(cluster.0)?;
    } else {
        w.write_u8(RECORD_TYPE_RANGE)?;
        w.write_u24::<LittleEndian>(cluster.0)?;
        w.write_u24::<LittleEndian>(cluster.1)?;
    }
    Ok(())
}

fn read_cluster<R: Read>(mut r: R) -> Result<Cluster> {
    let ty = r.read_u8()?;
    match ty {
        RECORD_TYPE_SINGLE => {
            let id = r.read_u24::<LittleEndian>()?;
            Ok(Cluster(id, id))
        }
        RECORD_TYPE_RANGE => Ok(Cluster(
            r.read_u24::<LittleEndian>()?,
            r.read_u24::<LittleEndian>()?,
        )),
        _ => Err(Error::new(
            ErrorKind::Other,
            format!("Unexpected record type {:?}", ty),
        )),
    }
}

fn encode<W: Write, I>(packets: I, mut w: W) -> Result<()>
where
    I: Iterator<Item = PacketNum>,
{
    let history = cluster(packets);

    w.write_u16::<BigEndian>(history.len() as u16)?;
    for cluster in history {
        write_cluster(cluster, &mut w)?;
    }

    Ok(())
}

fn decode<R: Read>(mut r: R) -> Result<Vec<PacketNum>> {
    let len = r.read_u16::<BigEndian>()?;
    let mut vec = Vec::with_capacity(len as usize);
    for _ in 0..len {
        let cluster = read_cluster(&mut r)?;
        for i in cluster.0..=cluster.1 {
            vec.push(i);
        }
    }
    Ok(vec)
}

#[derive(Clone, Debug, PartialEq)]
struct AckNack(Vec<PacketNum>);

impl CanIo for AckNack {
    fn write<W: Write>(&self, w: W) -> Result<()> {
        encode(self.0.iter().map(|&i| i), w)
    }

    fn read<R: Read>(r: R) -> Result<Self> {
        Ok(Self(decode(r)?))
    }
}

/// Acknowledges that datagrams are received
#[derive(Clone, Debug, rakrs_codegen::Packet, PartialEq)]
pub struct Ack(AckNack);

impl Ack {
    /// Creates an `Ack` packet.
    pub fn new(vec: Vec<PacketNum>) -> Self {
        Ack(AckNack(vec))
    }

    /// Retrieves the sequence numbers of datagrams acknowledged in this packet
    pub fn packets(&self) -> &Vec<PacketNum> {
        &self.0 .0
    }
}

/// Acknowledges that datagrams are missed
#[derive(Clone, Debug, rakrs_codegen::Packet, PartialEq)]
pub struct Nack(AckNack);

impl Nack {
    /// Creates a `Nack` packet.
    pub fn new(vec: Vec<PacketNum>) -> Self {
        Nack(AckNack(vec))
    }

    /// Retrieves the sequence numbers of datagrams unacknowledged in this packet
    pub fn packets(&self) -> &Vec<PacketNum> {
        &self.0 .0
    }
}
