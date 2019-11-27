#![feature(arbitrary_enum_discriminant, proc_macro_hygiene)]

pub use magic::Magic;
pub use offline::OfflinePacket;
pub use online::OnlinePacket;

pub mod encap;
mod magic;
pub mod offline;
pub mod online;
