//! Dead simple snapshot testing.

extern crate bincode;
extern crate diff;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate serde;
#[macro_use]
extern crate skittles;
#[macro_use]
extern crate trail;

pub mod test;

#[macro_use]
mod macros;
mod report;
mod store;

pub use failure::Error;
pub use report::Report;
pub use test::Test;

/// A convenient wrapper around [`Result`].
/// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
pub type Result<T> = ::std::result::Result<T, Error>;
