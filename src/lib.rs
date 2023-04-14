mod error;
#[macro_use]
pub(crate) mod utils;

pub mod dex;
pub mod raw;

pub(crate) type Result<T> = std::result::Result<T, error::Error>;
