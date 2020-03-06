use bytes::{BufMut, BytesMut};
use ring::aead::{
    Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, CHACHA20_POLY1305,
};
use ring::error::Unspecified;

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

#[derive(Debug, Clone)]
pub struct AeadCryptor {
    pub key: [u8; 32],
    pub tag_len: usize,
}

impl AeadCryptor {
    pub fn new(key_in: String) -> Self {
        let mut key_vec = key_in.as_bytes().to_vec();
        if key_vec.len() > 32 {
            key_vec = key_vec[..32].to_vec();
        } else {
            for _ in key_vec.len()..32 {
                key_vec.push(0);
            }
        };
        let mut key = [0; 32]; // 32 bit
        key.copy_from_slice(&key_vec);
        Self {
            key: key,
            tag_len: CHACHA20_POLY1305.tag_len(),
        }
    }
    pub fn encrypt(&self, input: &[u8], output: &mut [u8]) {
        let mut sealing_key = SealingKey::new(
            UnboundKey::new(&CHACHA20_POLY1305, &self.key).unwrap(),
            RingAeadNonceSequence::new(),
        );
        let mut buf = BytesMut::with_capacity(output.len());
        buf.put_slice(input);
        sealing_key
            .seal_in_place_append_tag(Aad::empty(), &mut buf)
            .unwrap();
        output.copy_from_slice(&buf[..output.len()]);
    }
    pub fn decrypt(&self, input: &[u8], output: &mut [u8]) {
        let mut opening_key = OpeningKey::new(
            UnboundKey::new(&CHACHA20_POLY1305, &self.key).unwrap(),
            RingAeadNonceSequence::new(),
        );
        let mut buf = BytesMut::with_capacity(input.len());
        buf.put_slice(input);
        let decrypted = opening_key.open_in_place(Aad::empty(), &mut buf).unwrap();
        output.copy_from_slice(&decrypted[..output.len()]);
    }
}
