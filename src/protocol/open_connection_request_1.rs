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

use std::io::{Read, Result, Write};

use super::CanIo;
use super::magic::Magic;

pub struct OpenConnectionRequest1 {
    pub magic: Magic,
    pub protocol: u8,
    pub mtu_size: usize,
}

impl CanIo for OpenConnectionRequest1 {
    fn write<W: Write>(&self, mut w: W) -> Result<()> {
        self.magic.write(&mut w)?;
        self.protocol.write(&mut w)?;

        w.write_all(&mut vec![0u8; self.mtu_size])?;
        Ok(())
    }

    fn read<R: Read>(mut r: R) -> Result<Self> {
        let magic = <Magic as CanIo>::read(&mut r)?;
        let protocol = <u8 as CanIo>::read(&mut r)?;
        let mtu_size = r.bytes().count();

        Ok(Self { magic, protocol, mtu_size })
    }
}
