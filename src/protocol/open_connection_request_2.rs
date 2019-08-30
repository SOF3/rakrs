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

use std::net::SocketAddr;

use super::magic::Magic;

#[derive(Packet)]
pub struct OpenConnectionRequest2 {
    pub magic: Magic,
    pub server_address: SocketAddr,
    pub mtu_size: u16,
    pub client_id: u64,
}
