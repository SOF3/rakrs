#[macro_export]
macro_rules! check_canio {
    ($read_name:ident : $($buf:literal)* = $write_name:ident : $expr:expr) => {
        #[test]
        pub fn $read_name() {
            let actual = ::rakrs_io::CanIo::read(::std::io::Cursor::new(vec![$($buf),*]))
                .expect("Panic reading vector cursor");
            assert_eq!($expr, actual);
        }

        #[test]
        pub fn $write_name() {
            let mut actual = ::std::vec::Vec::<u8>::new();
            ::rakrs_io::CanIo::write(&$expr, &mut actual).expect("Panic writing vector");
            assert_eq!(vec![$($buf),*], actual);
        }
    };
}
