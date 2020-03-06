#[macro_use]
extern crate log;

pub mod crypto;
pub mod error;
pub mod handler;
pub mod model;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
