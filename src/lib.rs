extern crate bincode;
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate serde;
#[macro_use]
extern crate skittles;
#[macro_use]
extern crate trail;

mod diff;
mod store;
mod test;

pub use failure::Error;
pub use store::Store;
pub use test::Test;

pub type Result<T> = ::std::result::Result<T, Error>;

#[macro_export]
macro_rules! snaptest {
    ( @items, $($i:item)* ) => { $($i)* };

    (
        $(#[$m:meta])*
        fn $name:ident() -> Result<$ret:ty, $err:ty> { $($code:tt)* }
        $($rest:tt)*
    ) => {
        snaptest!(@items,
            #[test]
            $(#[$m])*
            fn $name() {
                fn run() -> Result<$ret, $crate::Error> {
                    Ok({ $($code)* }?)
                }

                let test = $crate::Test::builder()
                    .file(file!())
                    .ident(stringify!($name))
                    .module(module_path!())
                    .ret_ty(concat!(
                        "Result<",
                        stringify!($ret),
                        ", ",
                        stringify!($err),
                        ">"
                    ))
                    .runner(run)
                    .build();

                match test.run() {
                    Ok(_) => (),
                    Err(e) => panic!("{}", e),
                }
            }
        );

        snaptest!($($rest)*);
    };

    (
        $(#[$m:meta])*
        fn $name:ident() -> $ret:ty { $($code:tt)* }
        $($rest:tt)*
    ) => {
        snaptest!(@items,
            #[test]
            $(#[$m])*
            fn $name() {
                fn run() -> Result<$ret, $crate::Error> {
                    Ok({ $($code)* })
                }

                let test = $crate::Test::builder()
                    .file(file!())
                    .ident(stringify!($name))
                    .module(module_path!())
                    .ret_ty(stringify!($ret))
                    .runner(run)
                    .build();

                match test.run() {
                    Ok(_) => (),
                    Err(e) => panic!("{}", e),
                }
            }
        );

        snaptest!($($rest)*);
    };

    ( $($rest:tt)* ) => ();
}

#[doc(hidden)]
#[macro_export]
macro_rules! __snaptest {
    ( @items $($i:item)* ) => { $($i)* };
    ( $test:ident, $ret:expr, $runner:expr ) => ({
        let test = $crate::Test::builder()
            .file(file!())
            .ident(stringify!($test))
            .module(module_path!())
            .ret_ty($ret)
            .runner($runner)
            .build();

        assert!(test.run().is_ok());
    });
}
