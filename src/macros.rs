/// A macro for writing snapshot tests.
///
/// ```rust
/// # #[macro_use]
/// # extern crate snaptest;
/// #
/// # fn main() {}
/// #
/// use std::str::FromStr;
///
/// #[derive(Debug)]
/// enum Hero {
///     Batman,
///     TheFlash,
///     WonderWoman,
/// }
///
/// impl FromStr for Hero {
/// #   type Err = ::snaptest::Error;
/// #
/// #    fn from_str(value: &str) -> Result<Hero, Self::Err> {
/// #       unimplemented!()
/// #    }
///     // ...
/// }
///
/// #[cfg(test)]
/// mod tests {
///     use super::Hero; // no pun intended...
///
///     snaptest!{
///         fn parse_heros() -> Result<Vec<Hero>, Error> {
///             let heros = ["Wonder Woman", "Batman", "The Flash"];
///             heros.iter().map(|hero| hero.parse()).collect()
///         }
///     }
/// }
/// #
/// ```
///
#[macro_export]
macro_rules! snaptest {
    ( $($test:tt)* ) => { __snaptest!($($test)*); }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __snaptest {
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
                    let result: Result<$ret, $err> = { $($code)* };
                    Ok(result?)
                }

                let result = $crate::Test::builder()
                    .file(file!())
                    .name(stringify!($name))
                    .path(module_path!())
                    .uuid(concat!(file!(), ":", stringify!($name)))
                    .ret(concat!(
                        "Result<",
                        stringify!($ret),
                        ", ",
                        stringify!($err),
                        ">"
                    ))
                    .run(run);

                match result {
                    Ok(ref report) if report.outcome().was_successful() => (),
                    Ok(ref report) => panic!("{}", report),
                    Err(ref e) => panic!("{}", e),
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

                let result = $crate::Test::builder()
                    .file(file!())
                    .name(stringify!($name))
                    .path(module_path!())
                    .uuid(concat!(file!(), ":", stringify!($name)))
                    .ret(stringify!($ret))
                    .run(run);

                match result {
                    Ok(ref report) if report.outcome().was_successful() => (),
                    Ok(ref report) => panic!("{}", report),
                    Err(ref e) => panic!("{}", e),
                }
            }
        );

        snaptest!($($rest)*);
    };

    ( @items, $($i:item)* ) => { $($i)* };
    ( $($rest:tt)* ) => {};
}
