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

pub use can_io::{CanIo, Little};

macro_rules! packets {
    ($($mod:ident $name:ident $id:literal;)*) => {
        $(pub use $mod::$name;)*

        #[repr(u8)]
        pub enum Packet { $($name($mod::$name) = $id),* }

        impl Packet {
            pub fn write<W: Write>(&self, mut w: W) -> Result<()> {
                match self {
                    $(
                        Packet::$name(var) => {
                            w.write_all(&[$id])?;
                            var.write(w)
                        },
                    )*
                }
            }

            pub fn read<R: Read>(mut r: R) -> Result<Option<Packet>> {
                let mut id = [0u8];
                r.read_exact(&mut id)?;
                match id[0] {
                    $(
                        $id => Ok(Some(Packet::$name($mod::$name::read(r)?))),
                    )*
                    _ => Ok(None),
                }
            }
        }
    };
}

mod ack;
mod advertise_system;
mod can_io;

packets! [
    ack Ack 0xc0;
    ack Nack 0xa0;
    advertise_system AdvertiseSystem 0x1d;
];

// Required methods on packets:
// fn read<R: Read>(r: R) -> Result<Self>;
// fn write<W: Write>(&self, w: W) -> Result<()>;
