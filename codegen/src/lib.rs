extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod packet;

/// Generate `rakrs_io::CanIo` implementation for structs and enums that have all fields implement `CanIo`
///
/// For structs, fields are written one by one in order.
///
/// For enums, the structure starts with a discriminant with the type specified in the `#[repr]` of
/// the enum, followed by the fields of the enum one by one. If the enum repr should be little
/// endian, the `#[little_endian]` attribute must be applied on the `enum` item.
#[proc_macro_derive(Packet, attributes(little_endian))]
pub fn derive_packet(item: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(item as DeriveInput);
    match packet::imp(parsed) {
        Ok(item) => item.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
