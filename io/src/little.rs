use derive_more::*;

/// A wrapper of the primitive types, encoded in little-endian instead of big-endian.
#[derive(Clone, Copy, Debug, Default, From, PartialEq, Eq, PartialOrd, Ord)]
pub struct Little<T: Copy + Default>(pub T);

impl<T: Copy + Default> Little<T> {
    #[inline]
    pub fn inner(self) -> T {
        self.0
    }
}
