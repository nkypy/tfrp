#[macro_use]
extern crate log;

use frp::error::Error;
use frp::Result;
use futures::TryFutureExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use serde::Serialize;
use std::convert::Infallible;
use std::net::SocketAddr;

use clap::Clap;
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio::stream::StreamExt;
use tokio_tungstenite::connect_async;
use tungstenite::protocol::Message;
use futures_util::sink::SinkExt;

#[derive(Clap)]
#[clap(name = "tfrpc", version = "0.1.0", author = "Jack Shih")]
struct Opts {
    #[clap(short = "c", long = "config", default_value = "config/tfrpc.toml")]
    config: String,
}

#[derive(Deserialize)]
struct Config {
    common: CommonConfig,
}

#[derive(Deserialize)]
struct CommonConfig {
    server_addr: String,
    server_port: u16,
    log_level: String,
    auth_token: String,
}

#[derive(Deserialize)]
enum ClientType {
    TCP,
    UDP,
}


#[derive(Deserialize)]
struct ClientConfig {
    client_type: ClientType,
    local_ip: String,
    local_port: u16,
    remote_port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Clap::parse();
    env_logger::init();
    let buf = std::fs::read_to_string(opts.config)?;
    let conf: Config = toml::from_str(&buf)?;
    // let conn = TcpStream::connect(format!("{}:{}",conf.common.server_addr, conf.common.server_port)).await?;
    let (mut ws_stream, _) = connect_async(format!("ws://{}:{}/clients",conf.common.server_addr, conf.common.server_port)).await?;
    ws_stream.send(Message::binary(buf.as_bytes())).await?;
    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        info!("client msg is {}", &msg);
        if msg.is_text() || msg.is_binary() {
            ws_stream.send(msg).await?;
        }
    }
    Ok(())
}