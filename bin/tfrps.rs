// Copyright (c) 2020 [Jack Shih <i@kshih.com>]
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fs;

use clap::Clap;

use tfrp::conn::server::Server;
use tfrp::{Result, ServerConfig};

#[derive(Clap)]
#[clap(name = "tfrps", version = tfrp::VERSION, author = tfrp::AUTHOR)]
struct Opts {
    #[clap(short = 'c', long = "config", default_value = "config/tfrps.toml")]
    config: String,
}

// async fn transfer(mut inbound: TcpStream, proxy_addr: String) -> Result<()> {
//     let mut outbound = TcpStream::connect(proxy_addr).await?;

//     let (mut ri, mut wi) = inbound.split();
//     let (mut ro, mut wo) = outbound.split();

//     let client_to_server = io::copy(&mut ri, &mut wo);
//     let server_to_client = io::copy(&mut ro, &mut wi);

//     try_join(client_to_server, server_to_client).await?;

//     Ok(())
// }

// #[cfg(target_arch = "aarch64")]
// fn test_cfg() -> () {
//     println!("this is aarch64");
// }

// #[cfg(not(target_arch = "aarch64"))]
// fn test_cfg() -> () {
//     println!("this is not aarch64");
// }

// async fn client_tcp_handle(
//     local_ip: String,
//     local_port: u16,
//     remote_port: u16,
//     mut rx: Receiver<()>,
// ) -> Result<()> {
//     info!(
//         "local tcp client {}:{}, remote port {}.",
//         &local_ip, &local_port, &remote_port
//     );
//     // let addr = SocketAddr::from(([127, 0, 0, 1], remote_port));
//     // let mut listener = TcpListener::bind(&addr).await?;
//     // // listener.set_nonblocking(true).expect("Cannot set non-blocking");
//     // while let Some(stream) = listener.next().await {
//     //     match stream {
//     //         Ok(stream) => {
//     //             // tokio::task::spawn(move|| {
//     //             //     // connection succeeded
//     //                 println!("new client!");
//     //             // });
//     //         }
//     //         Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
//     //             // Decide if we should exit
//     //             break;
//     //             // Decide if we should try to accept a connection again
//     //             continue;
//     //         }
//     //         Err(e) => { error!("TCP handle error {}", e); }
//     //     }
//     // }
//     info!("drop TCP handle client");
//     // drop(listener);
//     Ok(())
// }

// async fn client_udp_handle(
//     local_ip: String,
//     local_port: u16,
//     remote_port: u16,
//     mut rx: Receiver<()>,
// ) -> Result<()> {
//     info!(
//         "local udp client {}:{}, remote port {}.",
//         &local_ip, &local_port, &remote_port
//     );
//     // let addr = SocketAddr::from(([127, 0, 0, 1], remote_port));
//     // let mut socket = tokio::net::UdpSocket::bind(&addr).await?;
//     // let mut buf = [0u8;100];
//     // while let (amt, _addr) = socket.recv_from(&mut buf).await.expect("no data received") {
//     //     info!("UDP receive: {}", String::from_utf8_lossy(&buf[..amt]));
//     //     info!("UDP TODO");
//     // };
//     Ok(())
// }

// async fn client_http_handle(
//     local_ip: String,
//     local_port: u16,
//     remote_port: u16,
//     mut rx: Receiver<()>,
// ) -> Result<()> {
//     info!(
//         "local http client {}:{}, remote port {}.",
//         &local_ip, &local_port, &remote_port
//     );
//     // let addr = SocketAddr::from(([127, 0, 0, 1], remote_port));
//     // let new_service = make_service_fn(move |_| {
//     //     let name = format!("http://{}:{}", local_ip, local_port);
//     //     async { Ok::<_, hyper::Error>(service_fn(move |req| handle_request(req, name.to_owned()))) }
//     // });
//     // // 后续如果客户端断开，需要关闭服务端
//     // let srv = Server::bind(&addr).serve(new_service).with_graceful_shutdown(async {
//     //     rx.recv().await.unwrap();
//     // });
//     // srv.await?;
//     Ok(())
// }

// async fn handle_ws(
//     mut ws_stream: WebSocketStream<hyper::upgrade::Upgraded>,
//     rx: Receiver<()>,
// ) -> Result<()> {
//     // while let Some(msg) = ws_stream.next().await {
//     //     let msg = msg?;
//     //     info!("websocket msg is \n{}", msg);
//     //     let conf: std::result::Result<HashMap<String, tfrp::model::config::ClientConfig>, toml::de::Error> = toml::from_slice(&msg.into_data());
//     //     match conf {
//     //         Ok(c) => {
//     //             info!("clients conf from ws");
//     //             for i in c {
//     //                 let rx = rx.clone();
//     //                 match i.1.client_type {
//     //                     ClientType::TCP => {
//     //                         tokio::task::spawn(client_tcp_handle(i.1.local_ip, i.1.local_port, i.1.remote_port, rx));
//     //                     },
//     //                     ClientType::UDP => {
//     //                         tokio::task::spawn(client_udp_handle(i.1.local_ip, i.1.local_port, i.1.remote_port, rx));
//     //                     }
//     //                     ClientType::HTTP => {
//     //                         tokio::task::spawn(client_http_handle(i.1.local_ip, i.1.local_port, i.1.remote_port, rx));
//     //                     },
//     //                     _ => {debug!("TODO")},
//     //                 }

//     //             }
//     //         },
//     //         Err(_e) => {ws_stream.send(protocol::Message::binary("client config error")).await?;},
//     //     };
//     // };
//     Ok(())
// }

// async fn handle_conn(req: Request<Body>) -> Result<Response<Body>> {
//     // if !req.headers().contains_key(UPGRADE) || !req.uri().eq("/clients"){
//     //     return Ok(Error{}.into());
//     // };
//     // let key = req.headers().typed_get::<headers::SecWebsocketKey>();
//     // tokio::task::spawn(async move {
//     //     match req.into_body().on_upgrade().await {
//     //         Ok(upgraded) => {
//     //             let (tx, mut rx) = tokio::sync::watch::channel(());
//     //             rx.changed().await.unwrap();
//     //             let ws_stream = WebSocketStream::from_raw_socket(upgraded, protocol::Role::Server, None).await;
//     //             let mut app = tfrp::model::server::AppServer::new(ws_stream, tx, rx);
//     //             // debug!("app is {:?}", &app);
//     //             if let Err(e) = app.ws_handle().await {
//     //                 error!("handle websocket error: {}", e);
//     //                 let _ = app.tx.broadcast(());
//     //                 info!("shut down clients");
//     //             };
//     //             // if let Err(e) = handle_ws(app.ws, rx).await {
//     //             //     error!("handle websocket error: {}", e);
//     //             //     let _ = app.tx.broadcast(());
//     //             //     info!("shut down clients");
//     //             // };
//     //         }
//     //         Err(e) => error!("upgrade error: {}", e),
//     //     }
//     // });
//     let mut res = Response::new(Body::empty());
//     // websocket 返回头部
//     *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
//     let h = res.headers_mut();
//     h.typed_insert(headers::Upgrade::websocket());
//     // h.typed_insert(headers::SecWebsocketAccept::from(key.unwrap()));
//     h.typed_insert(headers::Connection::upgrade());
//     Ok(res)
// }

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Clap::parse();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let conf: ServerConfig = toml::from_str(&fs::read_to_string(opts.config)?)?;
    let addr = format!("0.0.0.0:{}", conf.common.bind_port);
    let server =
        Server::with_codec(tfrp::codec::AES128GCMCodec::new(conf.common.auth_token)).listen(addr);
    server.await?;
    Ok(())
}
