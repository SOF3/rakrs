#[macro_export]
macro_rules! canio_ok {
    ($read_name:ident : $($buf:literal),* $(,)? = $write_name:ident : $expr:expr) => {
        #[test]
        pub fn $read_name() {
            let actual = ::rakrs_io::CanIo::read(::std::io::Cursor::new(::std::vec![$($buf),*]))
                .expect("Panic reading correct data");
            assert_eq!($expr, actual);
        }

        #[test]
        pub fn $write_name() {
            let mut actual = ::std::vec::Vec::<u8>::new();
            ::rakrs_io::CanIo::write(&$expr, &mut actual).expect("Panic writing data");
            assert_eq!(vec![$($buf),*], actual);
        }
    };
}

#[macro_export]
macro_rules! canio_err_read {
    ($name:ident: $ty:ty => $err:literal; $($buf:literal),* $(,)?) => {
        #[test]
        pub fn $name() {
            let actual = <$ty as ::rakrs_io::CanIo>::read(::std::io::Cursor::new(::std::vec![$($buf),*]));
            match actual {
                Ok(_) => panic!("Read invalid data as valid"),
                Err(err) => assert_eq!($err, err.to_string().as_str()),
            }
        }
    };
}
