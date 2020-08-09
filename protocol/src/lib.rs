macro_rules! can_io {
    (struct $ident:ident {$(
        $vis:vis $field:ident : $ty:ty,
    )*}) => {
        #[derive(Debug, Clone)]
        pub struct $ident {
            $(
                $vis $field: $ty,
            )*
        }

        #[allow(unused_variables)]
        impl rakrs_io::CanIo for $ident {
            fn write(&self, vec: &mut Vec<u8>) {
                $(
                    rakrs_io::CanIo::write(&self.$field, &mut *vec);
                )*
            }

            fn read(src: &[u8], offset: &mut usize) -> Result<Self, rakrs_io::DecodeError> {
                Ok(Self {
                    $(
                        $field: rakrs_io::CanIo::read(src, &mut *offset)?,
                    )*
                })
            }
        }
    };
    (enum $ident:ident : $disty:ty {$(
        $var:ident = $disc:literal,
    )*}) => {
        #[derive(Debug, Clone)]
        pub enum $ident {
            $(
                $var($var),
            )*
        }

        impl rakrs_io::CanIo for $ident {
            fn write(&self, vec: &mut Vec<u8>) {
                match self {
                    $(
                        Self::$var(value) => {
                            <$disty as rakrs_io::CanIo>::write(&$disc, &mut *vec);
                            rakrs_io::CanIo::write(value, &mut *vec);
                        },
                    )*
                }
            }

            fn read(src: &[u8], offset: &mut usize) -> Result<Self, rakrs_io::DecodeError> {
                match <$disty as rakrs_io::CanIo>::read(src, &mut *offset)? {
                    $(
                        $disc => Ok(Self::$var(rakrs_io::CanIo::read(src, &mut *offset)?)),
                    )*
                    _ => Err(rakrs_io::DecodeError::OutOfRange),
                }
            }
        }
    };
}

mod magic;
pub use magic::Magic;

mod offline;
pub use offline::*;

mod reliable;
pub use reliable::*;

mod online;
pub use online::*;

pub mod inner;
pub use inner::Inner;
