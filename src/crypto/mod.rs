pub mod aead;

use crate::Result;
pub trait CryptoExt {
    fn encrypt(_src: Vec<u8>) -> Result<Vec<u8>>;
    fn decrypt(_encoded: Vec<u8>) -> Result<Vec<u8>>;
}
