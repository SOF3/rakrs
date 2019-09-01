// rakrs
// Copyright (C) SOFe
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[allow(unused_imports)]
use crate::prelude::*;

use std::io::{Cursor, Read, Result, Write};

use super::inner::InnerPacket;
use rakrs_io::{CanIo, Little, Triad};

#[derive(Clone, Debug)]
pub struct Datagram {
    pub packets: Vec<InnerPacket>,
    pub seq_number: Triad,
}

impl CanIo for Datagram {
    fn write<W: Write>(&self, mut w: W) -> Result<()> {
        Little(self.seq_number).write(&mut w)?;
        for packet in &self.packets {
            packet.write(&mut w)?;
        }
        Ok(())
    }

    fn read<R: Read>(mut r: R) -> Result<Self> {
        let seq_number = Little::<Triad>::read(&mut r)?.inner();

        let mut buf = vec![];
        r.read_to_end(&mut buf)?;
        drop(r);

        let mut packets = vec![];
        let mut cursor = Cursor::new(&buf);
        while (cursor.position() as usize) < buf.len() {
            packets.push(InnerPacket::read(&mut cursor)?);
        }

        Ok(Self {
            packets,
            seq_number,
        })
    }
}
