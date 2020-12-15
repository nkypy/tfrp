pub mod conn;
pub mod crypto;
pub mod error;
pub mod handler;
pub mod model;
pub mod protocol;

pub use crate::error::Error;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub const VERSION: &'static str = "0.1.0";
pub const AUTHOR: &'static str = "Jack Shih <i@kshih.com>";

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
