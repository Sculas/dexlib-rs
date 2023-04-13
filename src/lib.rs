mod error;
#[macro_use]
pub(crate) mod utils;
pub(crate) mod raw;

pub mod dex;

pub(crate) type Result<T> = std::result::Result<T, error::Error>;
