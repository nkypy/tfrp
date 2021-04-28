pub mod aead;

use crate::Result;
pub trait CodecExt {
    fn encode(&mut self, _src: Vec<u8>) -> Result<Vec<u8>>;
    fn decode(&mut self, _src: Vec<u8>) -> Result<Vec<u8>>;
}

pub struct BuiltInCodec;

impl CodecExt for BuiltInCodec {
    fn encode(&mut self, src: Vec<u8>) -> Result<Vec<u8>> {
        Ok(src)
    }
    fn decode(&mut self, src: Vec<u8>) -> Result<Vec<u8>> {
        Ok(src)
    }
}

pub struct TLSCodec;

impl CodecExt for TLSCodec {
    fn encode(&mut self, src: Vec<u8>) -> Result<Vec<u8>> {
        Ok(src)
    }
    fn decode(&mut self, src: Vec<u8>) -> Result<Vec<u8>> {
        Ok(src)
    }
}
