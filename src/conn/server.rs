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

use std::collections::HashMap;
use std::convert::TryFrom;

use futures::{Future, FutureExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{
    tcp::{ReadHalf, WriteHalf},
    TcpListener, TcpStream, ToSocketAddrs,
};
use tokio::sync::{mpsc, oneshot};

use crate::codec::{BuiltInCodec, CodecExt};
use crate::error::Error;
use crate::protocol::ProxyFrame;
use crate::Result;
// use crate::conn::client;

#[derive(Debug, Copy, Clone)]
pub struct Server<C: CodecExt> {
    codec: C,
    // rx: mpsc::Receiver<Vec<u8>>,
    // txp: mpsc::Sender<Vec<u8>>,
    // tx: HashMap<String, oneshot::Sender<Vec<u8>>>,
}

pub struct Proxy {
    tx: mpsc::Sender<Vec<u8>>,
    rx: oneshot::Receiver<Vec<u8>>,
}

impl Server<BuiltInCodec> {
    pub fn new() -> Self {
        Self::with_codec(BuiltInCodec {})
    }
}

impl Default for Server<BuiltInCodec> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> Server<C>
where
    C: CodecExt + Send + Sync + Copy + 'static,
{
    pub fn with_codec(codec: C) -> Self {
        // let (txp, rx) = mpsc::channel(32);
        Server {
            codec,
            // rx,
            // txp,
            // tx: HashMap::new(),
        }
    }
    pub async fn listen<A: ToSocketAddrs>(self, addr: A) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        while let Ok((stream, peer_addr)) = listener.accept().await {
            tracing::info!("accept connection from {}", peer_addr.to_string());
            Self::handle_tcp(stream, peer_addr.to_string(), async move |pa, src| {
                match ProxyFrame::try_from(src) {
                    Ok(frame) => {
                        match frame {
                            ProxyFrame::Conf(name, conf) => {
                                tracing::debug!(
                                    "inbound from {} read name {} config {:?}",
                                    name,
                                    pa,
                                    &conf
                                );
                                dbg!(self.codec.encode(name.as_bytes().to_vec()));
                                let addr = format!("0.0.0.0:{}", conf.remote_port);
                                tracing::debug!("proxy client {} is listening at {}", name, &addr);
                                Self::handle_proxy(name, addr);
                            }
                            ProxyFrame::Body(name, addr, body) => {
                                tracing::debug!("inbound from {} read body: {}", pa, name);
                            }
                        }
                        Some(())
                    }
                    Err(e) => {
                        tracing::error!("inbound from {} data serialize error {}", pa, e);
                        None
                    }
                }
            });
        }
        Ok(())
    }
    fn handle_tcp<F, Fut>(mut stream: TcpStream, peer_addr: String, func: F)
    where
        F: FnOnce(String, Vec<u8>) -> Fut + Send + Copy + 'static,
        Fut: Future<Output = Option<()>> + Send + 'static,
    {
        tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];
            loop {
                match stream.read(&mut buf).await {
                    Ok(0) => {
                        tracing::error!("inbound from {} read EOF", peer_addr);
                        break;
                    }
                    Ok(n) => {
                        match func(peer_addr.clone(), buf[0..n].to_vec()).await {
                            Some(_) => {}
                            None => {
                                if let Err(we) =
                                    stream.write(String::from(Error {}).as_bytes()).await
                                {
                                    tracing::error!(
                                        "inbound from {} write response error {}",
                                        peer_addr,
                                        we
                                    );
                                };
                            }
                        };
                    }
                    Err(e) => {
                        tracing::error!("inbound from {} read error {}", peer_addr, e);
                        break;
                    }
                }
            }
        });
    }
    fn handle_proxy(name: String, addr: String) {
        tokio::spawn(async move {
            let listener = TcpListener::bind(addr).await.unwrap();
            while let Ok((stream, peer_addr)) = listener.accept().await {
                tracing::info!("proxy accept connection from {}", peer_addr.to_string());
                Self::handle_tcp(stream, peer_addr.to_string(), async move |pa, src| None);
            }
        });
    }
}

// async fn transfer_stream_read(mut r: ReadHalf<'_>, mut tx: Sender<(bool, Vec<u8>)>) -> Result<()> {
//     let mut buf = vec![0u8; 4096];
//     let (mut btx, mut brx) = async_channel::unbounded();
//     loop {
//         match r.read(&mut buf).await {
//             Ok(0) => {
//                 tracing::error!("inbound read EOF");
//                 tx.send((false, vec![])).await?;
//                 break;
//             }
//             Ok(n) => {
//                 tracing::debug!("receive data size {}", n);
//                 // TODO: 分类数据
//                 if let Ok(frame) = buf[0..n].to_vec().try_into() {
//                     match frame {
//                         crate::protocol::ProxyFrame::Conf(name, conf) => {
//                             tracing::debug!("receive proxy client {:?}", &name);
//                             let addr = format!("0.0.0.0:{}", conf.remote_port);
//                             tracing::debug!("proxy server is listening at {}", &addr);
//                             let listen =
//                                 proxy_listen(name, addr, tx.clone(), brx.clone()).map(|r| {
//                                     if let Err(e) = r {
//                                         tracing::error!("proxy server bind error {}", e);
//                                     }
//                                 });
//                             tokio::spawn(listen);
//                         }
//                         crate::protocol::ProxyFrame::Body(name, addr, body) => {
//                             tracing::debug!("name: {}", name);
//                             btx.send((addr, body)).await?;
//                         }
//                     }
//                 } else {
//                     tx.send((false, vec![])).await?;
//                 };
//             }
//             Err(e) => {
//                 tracing::error!("inbound read error {}", e);
//                 break;
//             }
//         }
//     }
//     Ok(())
// }

// async fn transfer_stream_write(
//     mut w: WriteHalf<'_>,
//     mut rx: Receiver<(bool, Vec<u8>)>,
// ) -> Result<()> {
//     while let Some(message) = rx.recv().await {
//         // tracing::debug!("GOT = {:?}", &message);
//         if message.0 != true {
//             tracing::debug!("stream write not found page");
//             w.write(crate::error::HTTP_NOT_FOUND_HTML.as_bytes())
//                 .await?;
//         } else {
//             tracing::debug!("stream write data");
//             w.write(&message.1).await?;
//         }
//     }
//     Ok(())
// }

// async fn proxy_listen(
//     name: String,
//     addr: String,
//     tx: Sender<(bool, Vec<u8>)>,
//     brx: async_channel::Receiver<(String, Vec<u8>)>,
// ) -> Result<()> {
//     tracing::debug!("proxy listener bind to {}", &addr);
//     let listener = TcpListener::bind(addr).await?;
//     while let Ok((mut inbound, peer_addr)) = listener.accept().await {
//         tracing::info!("proxy stream addr {} is connected", &peer_addr);
//         let (rin, win) = inbound.split();
//         let peer = peer_addr.to_string();
//         tokio::try_join!(
//             proxy_listen_read(name.clone(), peer.clone(), rin, tx.clone()),
//             proxy_listen_write(win, peer, brx.clone())
//         )?;
//     }
//     Ok(())
// }

// async fn proxy_listen_read(
//     name: String,
//     peer_addr: String,
//     mut inbound: ReadHalf<'_>,
//     tx: Sender<(bool, Vec<u8>)>,
// ) -> Result<()> {
//     let mut buf = vec![0u8; 4096];
//     loop {
//         let name = name.clone();
//         match inbound.read(&mut buf).await {
//             Ok(0) => {
//                 tracing::error!("proxy stream disconnected");
//                 break;
//             }
//             Ok(n) => {
//                 tracing::debug!("proxy listen read size {}", n);
//                 let body =
//                     crate::protocol::ProxyFrame::Body(name, peer_addr.clone(), buf[0..n].to_vec());
//                 let encoded: Vec<u8> = bincode::serialize(&body).unwrap();
//                 tx.send((true, encoded)).await?;
//             }
//             Err(e) => {
//                 tracing::error!("proxy stream read error {}", e);
//                 break;
//             }
//         }
//     }
//     Ok(())
// }

// async fn proxy_listen_write(
//     mut inbound: WriteHalf<'_>,
//     peer_addr: String,
//     brx: async_channel::Receiver<(String, Vec<u8>)>,
// ) -> Result<()> {
//     while let Ok((addr, message)) = brx.recv().await {
//         tracing::debug!("proxy listen write get");
//         if peer_addr == addr {
//             inbound.write(&message).await?;
//         };
//     }
//     Ok(())
// }
