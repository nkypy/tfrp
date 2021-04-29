#![forbid(unsafe_code)]
#![feature(macro_attributes_in_derive_output)]
#![feature(async_closure)]
#![feature(test)]
extern crate test;

pub mod codec;
pub mod conn;
pub mod error;
pub mod model;
pub mod plugin;
pub mod protocol;
pub mod server;

pub use crate::error::Error;
pub use crate::model::config::{ClientConfig, ProxyClientConfig, ServerConfig};

pub type Result<T> = std::result::Result<T, anyhow::Error>;

pub const VERSION: &str = "0.1.0";
pub const AUTHOR: &str = "Jack Shih <i@kshih.com>";

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
