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

#![feature(proc_macro_span)]

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod packet;

#[proc_macro_derive(Packet, attributes(little_endian))]
pub fn derive_packet(item: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(item as DeriveInput);
    match packet::imp(parsed) {
        Ok(item) => item.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
