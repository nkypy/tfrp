#[macro_use]
extern crate log;

use tfrp::{Result, Error};
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
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::sink::SinkExt;
use std::collections::HashMap;

#[derive(Clap)]
#[clap(name = "tfrpc", version = "0.1.0", author = "Jack Shih <i@kshih.com>")]
struct Opts {
    #[clap(short = "c", long = "config", default_value = "config/tfrpc.toml")]
    config: String,
}

#[derive(Deserialize)]
struct Config {
    common: CommonConfig,
    clients: HashMap<String, tfrp::model::config::ClientConfig>,
}

#[derive(Deserialize)]
struct CommonConfig {
    server_addr: String,
    server_port: u16,
    log_level: String,
    auth_token: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Clap::parse();
    env_logger::init();
    let buf = std::fs::read_to_string(opts.config)?;
    let conf: Config = toml::from_str(&buf)?;
    // let conn = TcpStream::connect(format!("{}:{}",conf.common.server_addr, conf.common.server_port)).await?;
    let (mut ws_stream, _) = connect_async(format!("ws://{}:{}/clients",conf.common.server_addr, conf.common.server_port)).await?;
    ws_stream.send(Message::binary(toml::to_string(&conf.clients)?.as_bytes())).await?;
    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        info!("client msg is {}", &msg);
        if msg.is_text() || msg.is_binary() {
            info!("websocket client msg is {}", &msg);
            // ws_stream.send(msg).await?;
        }
    }
    Ok(())
}