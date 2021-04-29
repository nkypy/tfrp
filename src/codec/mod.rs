mod aead;
mod tls;

use crate::Result;

pub use self::aead::AES128GCMCodec;
pub use self::tls::TLSCodec;

pub trait CodecExt {
    fn encode(&self, _src: Vec<u8>) -> Result<Vec<u8>>;
    fn decode(&self, _src: Vec<u8>) -> Result<Vec<u8>>;
}

pub type BuiltInCodec = TLSCodec;
