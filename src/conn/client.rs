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

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{
    tcp::{ReadHalf, WriteHalf},
    TcpStream, ToSocketAddrs,
};
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::codec::{BuiltInCodec, CodecExt};
use crate::protocol::ProxyFrame;
use crate::Result;

#[derive(Debug)]
pub struct Client<C> {
    codec: C,
    rx: mpsc::Receiver<Vec<u8>>,
    txp: mpsc::Sender<Vec<u8>>,
    tx: HashMap<String, oneshot::Sender<Vec<u8>>>,
}

impl Client<BuiltInCodec> {
    pub fn new() -> Self {
        Self::with_codec(BuiltInCodec {})
    }
}

impl Default for Client<BuiltInCodec> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> Client<C>
where
    C: CodecExt,
{
    pub fn with_codec(codec: C) -> Self {
        let (txp, rx) = mpsc::channel(32);
        Self {
            codec,
            rx,
            txp,
            tx: HashMap::new(),
        }
    }
    pub async fn listen<A: ToSocketAddrs>(self, addr: A) -> Result<()> {
        let stream = TcpStream::connect(addr).await?;
        Self::handle_tcp(stream);
        Ok(())
    }
    fn handle_tcp(mut stream: TcpStream) {
        tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];
            loop {
                match stream.read(&mut buf).await {
                    Ok(0) => {
                        tracing::error!("inbound read EOF");
                        break;
                    }
                    Ok(n) => {
                        //             let decoded = bincode::deserialize::<ProxyFrame>(&buf[0..n])?;
                        //             dbg!(&decoded);
                        //             match decoded {
                        //                 ProxyFrame::Conf(name, conf) => {
                        //                     tracing::debug!("receive proxy client {:?}", &name);
                        //                     let addr = format!("0.0.0.0:{}", conf.remote_port);
                        //                     tracing::debug!("proxy server is listening at {}", &addr);
                        //                     let listener = TcpListener::bind(addr).await?;
                        //                     while let Ok((mut inbound, peer_addr)) = listener.accept().await {
                        //                         tracing::info!("proxy stream addr {} is connected", &peer_addr);
                        //                         let (rin, win) = inbound.split();
                        //                         let peer = peer_addr.to_string();
                        //                         tokio::try_join!(
                        //                             proxy_listen_read(name.clone(), peer.clone(), rin, tx.clone()),
                        //                             proxy_listen_write(win, peer, brx.clone())
                        //                         )?;
                        // }
                        //                     let listen =
                        //                         proxy_listen(name, addr, tx.clone(), brx.clone()).map(|r| {
                        //                             if let Err(e) = r {
                        //                                 tracing::error!("proxy server bind error {}", e);
                        //                             }
                        //                         });
                        //                     tokio::spawn(listen);
                        //                 },
                        //                 ProxyFrame::Body => {tx.send(decoded)?},
                        // }
                    }
                    Err(e) => {
                        tracing::error!("inbound read error {}", e);
                        break;
                    }
                }
            }
        });
    }
}

// pub async fn handle_read(mut r: ReadHalf<'_>, tx: broadcast::Sender<ProxyFrame>) -> crate::Result<()>{
//     let mut buf = vec![0u8; 4096];
//     loop {
//         match r.read(&mut buf).await {
//             Ok(0) => {
//                 tracing::error!("inbound read EOF");
//                 break;
//             }
//             Ok(n) => {
//                 let decoded = bincode::deserialize::<ProxyFrame>(&buf[0..n])?;
//                 dbg!(&decoded);
//                 match decoded {
//                     ProxyFrame::Conf(name, conf) => {
//                         tracing::debug!("receive proxy client {:?}", &name);
//                         let addr = format!("0.0.0.0:{}", conf.remote_port);
//                         tracing::debug!("proxy server is listening at {}", &addr);
//                         let listener = TcpListener::bind(addr).await?;
//                         while let Ok((mut inbound, peer_addr)) = listener.accept().await {
//                             tracing::info!("proxy stream addr {} is connected", &peer_addr);
//                             let (rin, win) = inbound.split();
//                             let peer = peer_addr.to_string();
//                             tokio::try_join!(
//                                 proxy_listen_read(name.clone(), peer.clone(), rin, tx.clone()),
//                                 proxy_listen_write(win, peer, brx.clone())
//                             )?;
//     }
//                         let listen =
//                             proxy_listen(name, addr, tx.clone(), brx.clone()).map(|r| {
//                                 if let Err(e) = r {
//                                     tracing::error!("proxy server bind error {}", e);
//                                 }
//                             });
//                         tokio::spawn(listen);
//                     },
//                     ProxyFrame::Body => {tx.send(decoded)?},
//                 }
//             }
//             Err(e) => {
//                 tracing::error!("inbound read error {}", e);
//                 break;
//             }
//         }
//     }
//     Ok(())
// }

pub async fn handle_write(
    mut w: WriteHalf<'_>,
    mut rx: mpsc::Receiver<ProxyFrame>,
) -> crate::Result<()> {
    while let Some(frame) = rx.recv().await {
        dbg!(&frame);
        let encoded = bincode::serialize(&frame)?;
        w.write(&encoded).await?;
    }
    Ok(())
}

pub async fn handle_transfer(
    mut conn: TcpStream,
    tx: mpsc::Sender<ProxyFrame>,
    mut rx: broadcast::Receiver<ProxyFrame>,
) -> crate::Result<()> {
    while let Ok(res) = rx.recv().await {
        if let ProxyFrame::Body(name, addr, body) = res {
            if let Some(data) = body {
                if let Ok(_) = conn.write(&data).await {
                    let mut buf = vec![0u8; 4096];
                    if let Ok(size) = conn.read(&mut buf).await {
                        tx.send(ProxyFrame::Body(name, addr, Some(buf[0..size].to_vec())))
                            .await?;
                        continue;
                    }
                }
            }
        }
        tx.send(ProxyFrame::Body("".to_string(), "".to_string(), None))
            .await?;
    }
    Ok(())
}
