use serde::{Deserialize, Serialize};

pub mod server;

#[derive(Debug, Serialize, Deserialize)]
pub enum ProxyFrame {
    Client(String, u16, u16),
    Body(String, String, Vec<u8>),
}
