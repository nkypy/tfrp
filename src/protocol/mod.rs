mod http;
mod tcp;
mod udp;

use crate::Result;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug)]
pub enum Protocol {
    TCP,
    UDP,
    HTTP,
    HTTPS,
    KCP,
    WS,
    WSS,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProxyFrame {
    Conf(String, u16, u16),        // 名称，本地端口，服务器端口
    Body(String, String, Vec<u8>), // 名称，连接地址，数据
}

impl TryFrom<Vec<u8>> for ProxyFrame {
    type Error = bincode::Error;

    fn try_from(value: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        bincode::deserialize::<Self>(&value)
    }
}

impl TryFrom<ProxyFrame> for Vec<u8> {
    type Error = bincode::Error;

    fn try_from(value: ProxyFrame) -> std::result::Result<Self, Self::Error> {
        bincode::serialize(&value)
    }
}

#[async_trait::async_trait]
pub trait ProxyClient {
    async fn name() -> String;
    async fn read() -> Result<()>;
    async fn write() -> Result<()>;
    async fn close();
}
