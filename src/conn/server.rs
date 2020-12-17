use futures::FutureExt;
use tokio::net::{
    tcp::{ReadHalf, WriteHalf},
    TcpListener, TcpStream,
};
use tokio::prelude::*;
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::Result;

pub async fn listen(addr: String) -> Result<()> {
    tracing::info!("tfrps bind to {}", &addr);
    let listener = TcpListener::bind(addr).await?;
    while let Ok((mut inbound, peer_addr)) = listener.accept().await {
        let transfer = transfer(inbound, peer_addr.to_string()).map(|r| {
            if let Err(e) = r {
                tracing::error!("Failed to transfer; error={}", e);
            }
        });
        tokio::spawn(transfer);
    }
    Ok(())
}

async fn transfer(mut inbound: TcpStream, peer_addr: String) -> Result<()> {
    tracing::debug!("accept stream from {}", peer_addr);
    let (r, w) = inbound.split();
    let (tx, rx) = mpsc::channel::<Vec<u8>>(32);
    tokio::try_join!(transfer_stream_read(r, tx), transfer_stream_write(w, rx))?;
    Ok(())
}

async fn transfer_stream_read(mut r: ReadHalf<'_>, tx: Sender<Vec<u8>>) -> Result<()> {
    let mut buf = vec![0u8; 1024];
    loop {
        match r.read(&mut buf).await {
            Ok(0) => {
                tracing::error!("inbound read EOF");
                break;
            }
            Ok(n) => {
                let proxy: crate::model::client::ProxyClient = toml::from_slice(&buf[0..n])?;
                tracing::debug!("receive proxy client {:?}", &proxy);
                let addr = format!("0.0.0.0:{}", proxy.remote_port);
                tracing::debug!("proxy server is listening at {}", &addr);
                let listen = proxy_listen(proxy.name, addr, tx.clone()).map(|r| {
                    if let Err(e) = r {
                        tracing::error!("proxy server bind error {}", e);
                    }
                });
                tokio::spawn(listen);
            }
            Err(e) => {
                tracing::error!("inbound read error {}", e);
                break;
            }
        }
    }
    Ok(())
}

async fn transfer_stream_write(mut w: WriteHalf<'_>, mut rx: Receiver<Vec<u8>>) -> Result<()> {
    while let Some(message) = rx.recv().await {
        tracing::debug!("GOT = {:?}", &message);
        w.write(&message).await?;
    }
    Ok(())
}

async fn proxy_listen(name: String, addr: String, tx: Sender<Vec<u8>>) -> Result<()> {
    tracing::debug!("proxy listener bind to {}", &addr);
    let listener = TcpListener::bind(addr).await?;
    while let Ok((mut inbound, peer_addr)) = listener.accept().await {
        tracing::info!("proxy stream addr {} is connected", peer_addr);
        loop {
            let mut buf = vec![0u8; 1024];
            let name = name.clone();
            match inbound.read(&mut buf).await {
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
                    tx.send(encoded).await?;
                }
                Err(e) => {
                    tracing::error!("proxy stream read error {}", e);
                    break;
                }
            }
        }
    }
    Ok(())
}
