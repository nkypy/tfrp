use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    pub server_addr: String,
    pub server_port: u16,
    pub clients: HashMap<String, ProxyClient>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyClient {
    pub name: String,
    pub local_port: u16,
    pub remote_port: u16,
}
