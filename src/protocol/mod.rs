mod tcp;

use crate::{ProxyClientConfig, Result};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyFrame {
    Conf(String, ProxyClientConfig),       // 名称，本地端口，服务器端口
    Body(String, String, Option<Vec<u8>>), // 名称，连接地址，数据(失败时为None)
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
