#![recursion_limit = "1024"]

#[macro_use]
extern crate log;

use clap::Clap;
use futures::future::{try_join_all, try_join};
use futures::FutureExt;
use futures_util::sink::SinkExt;
use futures_util::TryFutureExt;
use hyper::{Body, Client, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::header::{UPGRADE, HeaderValue};
use hyper_tls::HttpsConnector;
use headers::{self, HeaderMapExt};
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::prelude::*;
use tokio::stream::StreamExt;
use tokio::task;
use tokio::sync::watch::Receiver;
use tokio_tungstenite::{WebSocketStream, tungstenite::protocol};

use tfrp::{Result, Error};
use std::collections::HashMap;

#[derive(Clap)]
#[clap(name = "tfrps", version = "0.1.0", author = "Jack Shih <i@kshih.com>")]
struct Opts {
    #[clap(short = "c", long = "config", default_value = "config/tfrps.toml")]
    config: String,
}

#[derive(Deserialize)]
struct Config {
    common: CommonConfig,
}

#[derive(Deserialize)]
struct CommonConfig {
    bind_port: u16,
    auth_token: String,
}

async fn transfer(mut inbound: TcpStream, proxy_addr: String) -> Result<()> {
    let mut outbound = TcpStream::connect(proxy_addr).await?;

    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = io::copy(&mut ri, &mut wo);
    let server_to_client = io::copy(&mut ro, &mut wi);

    try_join(client_to_server, server_to_client).await?;

    Ok(())
}

#[cfg(target_arch = "aarch64")]
fn test_cfg() -> () {
    println!("this is aarch64");
}

#[cfg(not(target_arch = "aarch64"))]
fn test_cfg() -> () {
    println!("this is not aarch64");
}

async fn handle_request(
    req: Request<Body>,
    name: String,
) -> std::result::Result<Response<Body>, hyper::Error> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let url = req.uri().to_string();
    println!("url is {} method is {}", url, &req.method());
    let mut req = req;
    *req.uri_mut() = format!("{}{}", name, &url).parse::<hyper::Uri>().unwrap();
    // client.request(req).await
    let res = client.request(req).await;
    match res {
        Ok(body) => Ok(body),
        Err(e) => {
            error!("http client error {}", e);
            let e: Error = e.into();
            Ok(e.into())
        }
    }
}

async fn new_srv(name: String, port: u16, mut rx: Receiver<()>) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("listen new server of {}", &addr);
    let new_service = make_service_fn(move |_| {
        let name = name.clone();
        async { Ok::<_, hyper::Error>(service_fn(move |req| handle_request(req, name.to_owned()))) }
    });
    // 后续如果客户端断开，需要关闭服务端
    let srv = Server::bind(&addr).serve(new_service).with_graceful_shutdown(async {
        rx.recv().await.unwrap();
    });
    srv.await?;
    Ok(())
}

async fn handle_ws(upgraded: hyper::upgrade::Upgraded, rx: Receiver<()>) -> Result<()> {
    let mut ws_stream = WebSocketStream::from_raw_socket(upgraded, protocol::Role::Server, None).await;
    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        info!("websocket msg is \n{}", msg);
        let conf: std::result::Result<HashMap<String, tfrp::model::config::ClientConfig>, toml::de::Error> = toml::from_slice(&msg.into_data());
        match conf {
            Ok(c) => {
                info!("clients conf from ws");
                for i in c {
                    let mut rx = rx.clone();
                    tokio::task::spawn(new_srv(format!("http://{}:{}", i.1.local_ip, i.1.local_port), i.1.remote_port, rx));
                }
            },
            Err(_e) => {ws_stream.send(protocol::Message::binary("client config error")).await?;},
        };
    };
    Ok(())
}

async fn handle_conn(req: Request<Body>) -> Result<Response<Body>> {
    if !req.headers().contains_key(UPGRADE) || !req.uri().eq("/clients"){
        return Ok(Error{}.into());
    };
    let key = req.headers().typed_get::<headers::SecWebsocketKey>();
    tokio::task::spawn(async move {
        match req.into_body().on_upgrade().await {
            Ok(upgraded) => {
                let (tx, mut rx) = tokio::sync::watch::channel(());
                rx.recv().await.unwrap();
                if let Err(e) = handle_ws(upgraded, rx).await {
                    error!("handle websocket error: {}", e);
                    let _ = tx.broadcast(());
                    info!("shut down clients");
                };
            }
            Err(e) => error!("upgrade error: {}", e),
        }
    });
    let mut res = Response::new(Body::empty());
    // websocket 返回头部
    *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
    let h = res.headers_mut();
    h.typed_insert(headers::Upgrade::websocket());
    h.typed_insert(headers::SecWebsocketAccept::from(key.unwrap()));
    h.typed_insert(headers::Connection::upgrade());
    Ok(res)
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Clap::parse();
    env_logger::init();
    let buf = std::fs::read_to_string(opts.config)?;
    let conf: Config = toml::from_str(&buf)?;
    let new_service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(handle_conn)) });
    let srv = Server::bind(&SocketAddr::from(([0,0,0,0], conf.common.bind_port))).serve(new_service);
    info!("tfrp server is listening at 0.0.0.0:{}.", conf.common.bind_port);
    srv.await?;
    Ok(())
}
