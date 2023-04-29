// FIXME: Stable support => https://stackoverflow.com/a/43174171
#![allow(incomplete_features)]
#![feature(return_position_impl_trait_in_trait)]

mod error;
#[macro_use]
pub(crate) mod utils;

pub mod dex;
pub mod raw;

pub(crate) type Result<T> = std::result::Result<T, error::Error>;

#[cfg(test)]
pub(crate) mod t {
    macro_rules! dex {
        () => {
            crate::dex::DexFile::open(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/classes.dex"
            )))
            .unwrap()
        };
    }
    pub(crate) use dex;
}
