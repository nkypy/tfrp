use async_channel::{bounded, Receiver, Sender};
use async_net::{SocketAddr, TcpListener, TcpStream};
use futures::{io, prelude::*};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;

use crate::Result;

static PROXY_CLIENTS: Lazy<RwLock<HashMap<String, ProxyListener>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Debug)]
pub struct Listener {
    pub addr: String,
    pub listener: TcpListener,
    pub client_chan: (Sender<TcpStream>, Receiver<TcpStream>),
    pub proxy_chan: (
        Sender<(String, TcpStream, TcpListener)>,
        Receiver<(String, TcpStream, TcpListener)>,
    ),
}

#[derive(Debug)]
pub struct ProxyListener {
    pub name: String,
    pub addr: String,
    pub listener: TcpListener,
    pub stream: TcpStream,
}

impl Listener {
    pub async fn new(addr: String, port: u16) -> Result<Self> {
        let addr = format!("{}:{}", addr, port);
        let listener = TcpListener::bind(addr.clone()).await?;
        let client_chan = bounded::<TcpStream>(64);
        let proxy_chan = bounded::<(String, TcpStream, TcpListener)>(64);
        Ok(Self {
            addr,
            listener,
            client_chan,
            proxy_chan,
        })
    }

    pub async fn listen(&self) -> Result<()> {
        tracing::debug!("listener listen");
        let mut incoming = self.listener.incoming();
        while let Some(Ok(stream)) = incoming.next().await {
            tracing::info!("client stream addr {} is connected", stream.peer_addr()?);
            self.client_chan.0.send(stream).await?;
        }
        Ok(())
    }

    pub async fn write(&self) -> Result<()> {
        tracing::debug!("listener write");
        while let Ok(mut conn) = self.client_chan.1.recv().await {
            tokio::spawn(async move {
                let mut buf = vec![0u8; 1024];
                loop {
                    let size = conn.read(&mut buf).await.unwrap();
                    if size <= 0 {
                        break;
                    };
                    let proxy: crate::model::client::ProxyClient =
                        toml::from_slice(&buf[0..size]).unwrap();
                    tracing::debug!("receive proxy client {:?}", &proxy);
                    let addr = format!("127.0.0.1:{}", proxy.remote_port);
                    tracing::debug!("proxy server is listening at {}", &addr);
                    let mut conn2 = conn.clone();
                    tokio::spawn(async move {
                        let pl = ProxyListener::new(proxy.name, addr, conn2.clone())
                            .await
                            .unwrap();
                        pl.listen().await.unwrap();
                    });
                }
                tracing::error!(
                    "client stream addr {} is closed",
                    &conn.peer_addr().unwrap_or("127.0.0.1:80".parse().unwrap())
                );
            });
        }
        Ok(())
    }
}

impl ProxyListener {
    pub async fn new(name: String, addr: String, stream: TcpStream) -> Result<Self> {
        let listener = TcpListener::bind(addr.clone()).await?;
        Ok(Self {
            name,
            addr,
            listener,
            stream,
        })
    }

    pub async fn listen(&self) -> Result<()> {
        tracing::debug!("proxy listener {} listen", &self.addr);
        let mut incoming = self.listener.incoming();
        while let Some(Ok(mut stream)) = incoming.next().await {
            tracing::info!("proxy stream addr {} is connected", stream.peer_addr()?);
            let name = self.name.clone();
            loop {
                let mut conn = self.stream.clone();
                let mut buf = vec![0u8; 1024];
                let name = name.clone();
                match stream.read(&mut buf).await {
                    Ok(0) => {
                        tracing::error!("proxy stream disconnected");
                        break;
                    }
                    Ok(n) => {
                        let body = super::ProxyFrame {
                            name,
                            body: buf[0..n].to_vec(),
                            size: n,
                        };
                        let encoded: Vec<u8> = bincode::serialize(&body).unwrap();
                        // tracing::debug!("send data {:?}", &encoded);
                        conn.write(&encoded).await?;
                    }
                    Err(_) => {
                        tracing::error!("proxy stream read error");
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
