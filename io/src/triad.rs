use derive_more::*;

/// A wrapper over `u32`, with only the three least-significant bytes encoded.
#[derive(Clone, Copy, Debug, Default, From, Into, PartialEq, Eq, PartialOrd, Ord)]
pub struct Triad(u32); // TODO check overflow
