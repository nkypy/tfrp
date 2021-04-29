use ring::aead::{
    Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, AES_128_GCM,
};
use ring::error::Unspecified;

use super::CodecExt;
use crate::Result;

struct RingAeadNonceSequence {
    nonce: [u8; 12],
}

impl RingAeadNonceSequence {
    fn new() -> RingAeadNonceSequence {
        RingAeadNonceSequence { nonce: [0u8; 12] }
    }
}

impl NonceSequence for RingAeadNonceSequence {
    fn advance(&mut self) -> std::result::Result<Nonce, Unspecified> {
        let nonce = Nonce::assume_unique_for_key(self.nonce);
        Ok(nonce)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AES128GCMCodec {
    key: [u8; 16],
    tag_len: usize,
}

impl AES128GCMCodec {
    pub fn new(key_in: String) -> Self {
        let mut key = [0; 16];
        let keyb = key_in.as_bytes();
        for i in 0..keyb.len() {
            if i >= 16 {
                break;
            }
            key[i] = keyb[i];
        }
        Self {
            key,
            tag_len: AES_128_GCM.tag_len(),
        }
    }
}

impl CodecExt for AES128GCMCodec {
    fn encode(&self, input: Vec<u8>) -> Result<Vec<u8>> {
        let mut sealing_key = SealingKey::new(
            UnboundKey::new(&AES_128_GCM, &self.key).map_err(|e| anyhow::format_err!(e))?,
            RingAeadNonceSequence::new(),
        );
        let mut buf = input.clone();
        for _i in 0..self.tag_len {
            buf.push(0);
        }
        sealing_key
            .seal_in_place_append_tag(Aad::empty(), &mut buf)
            .map_err(|e| anyhow::format_err!(e))?;
        Ok(buf)
    }
    fn decode(&self, input: Vec<u8>) -> Result<Vec<u8>> {
        let mut opening_key = OpeningKey::new(
            UnboundKey::new(&AES_128_GCM, &self.key).map_err(|e| anyhow::format_err!(e))?,
            RingAeadNonceSequence::new(),
        );
        let mut buf = input.clone();
        let decrypted = opening_key
            .open_in_place(Aad::empty(), &mut buf)
            .map_err(|e| anyhow::format_err!(e))?;
        Ok(decrypted[..decrypted.len() - self.tag_len].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn test_aes_codec() {
        let codec = AES128GCMCodec::new("test key".to_string());
        let encrypted = codec.encode(b"test".to_vec()).unwrap();
        assert_eq!(b"test".to_vec(), codec.decode(encrypted).unwrap());
        ()
    }

    #[bench]
    fn bench_aes_encode(b: &mut Bencher) {
        let codec = AES128GCMCodec::new("test key".to_string());
        b.iter(|| codec.encode(b"test content".to_vec()).unwrap());
    }

    #[bench]
    fn bench_aes_decode(b: &mut Bencher) {
        let codec = AES128GCMCodec::new("test key".to_string());
        let encrypted = codec.encode(b"test content".to_vec()).unwrap();
        b.iter(|| codec.decode(encrypted.clone()).unwrap());
    }
}
