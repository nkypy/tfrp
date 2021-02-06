use async_net::{TcpListener, TcpStream};
use futures::prelude::*;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use serde::Serialize;
use std::convert::Infallible;
use std::fs;
use std::net::SocketAddr;
use tfrp::{Error, Result};

use clap::Clap;
use futures_util::sink::SinkExt;
use serde::Deserialize;
use std::collections::HashMap;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use tfrp::conn::ProxyFrame;

#[derive(Clap)]
#[clap(name = "tfrpc", version = tfrp::VERSION, author = tfrp::AUTHOR)]
struct Opts {
    #[clap(short = 'c', long = "config", default_value = "config/tfrpc.toml")]
    config: String,
}

#[derive(Deserialize)]
struct Config {
    common: CommonConfig,
    clients: HashMap<String, ClientConfig>,
}

#[derive(Deserialize)]
struct CommonConfig {
    server_addr: String,
    server_port: u16,
    log_level: String,
    auth_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    pub local_port: u16,
    pub remote_port: u16,
}

#[derive(Debug)]
pub struct Listener {
    pub conn: TcpStream,
    pub proxy: HashMap<String, TcpStream>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Clap::parse();
    tracing_subscriber::fmt::init();
    let buf = fs::read_to_string(opts.config)?;
    let conf: Config = toml::from_str(&buf)?;
    let addr = format!("{}:{}", conf.common.server_addr, conf.common.server_port);
    tracing::info!("tfrp client is connecting to {}", &addr);
    let mut listener = TcpStream::connect(addr).await?;
    tracing::debug!(
        "client stream locol addr is {}",
        &listener.local_addr().unwrap()
    );
    let mut l = Listener {
        conn: listener.clone(),
        proxy: HashMap::new(),
    };
    for (k, v) in conf.clients {
        let encoded =
            bincode::serialize(&ProxyFrame::Client(k.clone(), v.local_port, v.remote_port))
                .unwrap();
        tracing::debug!("send client config {:?}", &encoded);
        listener.write(&encoded).await?;
        let conn = TcpStream::connect(format!("127.0.0.1:{}", v.local_port)).await?;
        l.proxy.insert(k, conn.clone());
        tracing::debug!("proxy stream locol addr is {}", conn.local_addr().unwrap());
    }
    let conn = l.proxy.get_mut(&"tcp1".to_string()).unwrap();
    loop {
        let mut buf = vec![0u8; 5120];
        match listener.read(&mut buf).await {
            Ok(0) => {
                tracing::error!("read data EOF");
                break;
            }
            Ok(n) => {
                let decoded: ProxyFrame = bincode::deserialize(&buf[0..n]).unwrap();
                tracing::debug!("read from server size {}", n);
                if let ProxyFrame::Body(name, addr, body) = decoded {
                    conn.write(&body).await?;
                    let mut buf = vec![0u8; 5120];
                    let size = conn.read(&mut buf).await?;
                    // tracing::debug!("read new data {:?}", &buf[0..size]);
                    // TODO: 回传数据
                    let encoded =
                        bincode::serialize(&ProxyFrame::Body(name, addr, buf[0..size].to_vec()))
                            .unwrap();
                    listener.write(&encoded).await?;
                };
            }
            Err(e) => {
                tracing::error!("read data error {:?}", e);
                break;
            }
        };
        tracing::debug!("proxy end");
    }
    Ok(())
}
