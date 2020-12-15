use serde::{Deserialize, Serialize};

pub mod server;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyFrame {
    name: String,
    body: Vec<u8>,
    size: usize,
}
