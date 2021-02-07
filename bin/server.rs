#![recursion_limit = "1024"]

use clap::Clap;
use futures::future::{try_join, try_join_all};
use futures::FutureExt;
use futures_util::sink::SinkExt;
use futures_util::TryFutureExt;
use headers::{self, HeaderMapExt};
use hyper::header::{HeaderValue, UPGRADE};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server, StatusCode};
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
// use tokio::stream::StreamExt;
use tokio::sync::watch::Receiver;
use tokio::task;
use tokio_tungstenite::{tungstenite::protocol, WebSocketStream};
use tracing::{debug, error, info};

use std::collections::HashMap;
use tfrp::model::config::ClientProtocol;
use tfrp::{Error, Result};

#[derive(Clap)]
#[clap(name = "tfrps", version = tfrp::VERSION, author = tfrp::AUTHOR)]
struct Opts {
    #[clap(short = 'c', long = "config", default_value = "config/tfrps.toml")]
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

// async fn handle_request(
//     req: Request<Body>,
//     name: String,
// ) -> std::result::Result<Response<Body>, hyper::Error> {
//     let https = HttpsConnector::new();
//     let client = Client::builder().build::<_, hyper::Body>(https);
//     let url = req.uri().to_string();
//     println!("url is {} method is {}", url, &req.method());
//     let mut req = req;
//     *req.uri_mut() = format!("{}{}", name, &url).parse::<hyper::Uri>().unwrap();
//     // client.request(req).await
//     let res = client.request(req).await;
//     match res {
//         Ok(body) => Ok(body),
//         Err(e) => {
//             error!("http client error {}", e);
//             let e: Error = e.into();
//             Ok(e.into())
//         }
//     }
// }

async fn client_tcp_handle(
    local_ip: String,
    local_port: u16,
    remote_port: u16,
    mut rx: Receiver<()>,
) -> Result<()> {
    info!(
        "local tcp client {}:{}, remote port {}.",
        &local_ip, &local_port, &remote_port
    );
    // let addr = SocketAddr::from(([127, 0, 0, 1], remote_port));
    // let mut listener = TcpListener::bind(&addr).await?;
    // // listener.set_nonblocking(true).expect("Cannot set non-blocking");
    // while let Some(stream) = listener.next().await {
    //     match stream {
    //         Ok(stream) => {
    //             // tokio::task::spawn(move|| {
    //             //     // connection succeeded
    //                 println!("new client!");
    //             // });
    //         }
    //         Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
    //             // Decide if we should exit
    //             break;
    //             // Decide if we should try to accept a connection again
    //             continue;
    //         }
    //         Err(e) => { error!("TCP handle error {}", e); }
    //     }
    // }
    info!("drop TCP handle client");
    // drop(listener);
    Ok(())
}

async fn client_udp_handle(
    local_ip: String,
    local_port: u16,
    remote_port: u16,
    mut rx: Receiver<()>,
) -> Result<()> {
    info!(
        "local udp client {}:{}, remote port {}.",
        &local_ip, &local_port, &remote_port
    );
    // let addr = SocketAddr::from(([127, 0, 0, 1], remote_port));
    // let mut socket = tokio::net::UdpSocket::bind(&addr).await?;
    // let mut buf = [0u8;100];
    // while let (amt, _addr) = socket.recv_from(&mut buf).await.expect("no data received") {
    //     info!("UDP receive: {}", String::from_utf8_lossy(&buf[..amt]));
    //     info!("UDP TODO");
    // };
    Ok(())
}

async fn client_http_handle(
    local_ip: String,
    local_port: u16,
    remote_port: u16,
    mut rx: Receiver<()>,
) -> Result<()> {
    info!(
        "local http client {}:{}, remote port {}.",
        &local_ip, &local_port, &remote_port
    );
    // let addr = SocketAddr::from(([127, 0, 0, 1], remote_port));
    // let new_service = make_service_fn(move |_| {
    //     let name = format!("http://{}:{}", local_ip, local_port);
    //     async { Ok::<_, hyper::Error>(service_fn(move |req| handle_request(req, name.to_owned()))) }
    // });
    // // 后续如果客户端断开，需要关闭服务端
    // let srv = Server::bind(&addr).serve(new_service).with_graceful_shutdown(async {
    //     rx.recv().await.unwrap();
    // });
    // srv.await?;
    Ok(())
}

async fn handle_ws(
    mut ws_stream: WebSocketStream<hyper::upgrade::Upgraded>,
    rx: Receiver<()>,
) -> Result<()> {
    // while let Some(msg) = ws_stream.next().await {
    //     let msg = msg?;
    //     info!("websocket msg is \n{}", msg);
    //     let conf: std::result::Result<HashMap<String, tfrp::model::config::ClientConfig>, toml::de::Error> = toml::from_slice(&msg.into_data());
    //     match conf {
    //         Ok(c) => {
    //             info!("clients conf from ws");
    //             for i in c {
    //                 let rx = rx.clone();
    //                 match i.1.client_type {
    //                     ClientType::TCP => {
    //                         tokio::task::spawn(client_tcp_handle(i.1.local_ip, i.1.local_port, i.1.remote_port, rx));
    //                     },
    //                     ClientType::UDP => {
    //                         tokio::task::spawn(client_udp_handle(i.1.local_ip, i.1.local_port, i.1.remote_port, rx));
    //                     }
    //                     ClientType::HTTP => {
    //                         tokio::task::spawn(client_http_handle(i.1.local_ip, i.1.local_port, i.1.remote_port, rx));
    //                     },
    //                     _ => {debug!("TODO")},
    //                 }

    //             }
    //         },
    //         Err(_e) => {ws_stream.send(protocol::Message::binary("client config error")).await?;},
    //     };
    // };
    Ok(())
}

async fn handle_conn(req: Request<Body>) -> Result<Response<Body>> {
    // if !req.headers().contains_key(UPGRADE) || !req.uri().eq("/clients"){
    //     return Ok(Error{}.into());
    // };
    // let key = req.headers().typed_get::<headers::SecWebsocketKey>();
    // tokio::task::spawn(async move {
    //     match req.into_body().on_upgrade().await {
    //         Ok(upgraded) => {
    //             let (tx, mut rx) = tokio::sync::watch::channel(());
    //             rx.changed().await.unwrap();
    //             let ws_stream = WebSocketStream::from_raw_socket(upgraded, protocol::Role::Server, None).await;
    //             let mut app = tfrp::model::server::AppServer::new(ws_stream, tx, rx);
    //             // debug!("app is {:?}", &app);
    //             if let Err(e) = app.ws_handle().await {
    //                 error!("handle websocket error: {}", e);
    //                 let _ = app.tx.broadcast(());
    //                 info!("shut down clients");
    //             };
    //             // if let Err(e) = handle_ws(app.ws, rx).await {
    //             //     error!("handle websocket error: {}", e);
    //             //     let _ = app.tx.broadcast(());
    //             //     info!("shut down clients");
    //             // };
    //         }
    //         Err(e) => error!("upgrade error: {}", e),
    //     }
    // });
    let mut res = Response::new(Body::empty());
    // websocket 返回头部
    *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
    let h = res.headers_mut();
    h.typed_insert(headers::Upgrade::websocket());
    // h.typed_insert(headers::SecWebsocketAccept::from(key.unwrap()));
    h.typed_insert(headers::Connection::upgrade());
    Ok(res)
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Clap::parse();
    tracing_subscriber::fmt().pretty().init();
    let buf = std::fs::read_to_string(opts.config)?;
    let conf: Config = toml::from_str(&buf)?;
    let addr = format!("0.0.0.0:{}", conf.common.bind_port);
    tfrp::conn::server::listen(addr).await?;
    Ok(())
}
