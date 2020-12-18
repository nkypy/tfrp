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
    while let Ok((inbound, peer_addr)) = listener.accept().await {
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

async fn transfer_stream_read(mut r: ReadHalf<'_>, mut tx: Sender<Vec<u8>>) -> Result<()> {
    let mut buf = vec![0u8; 4096];
    let (mut btx, mut brx) = async_channel::unbounded();
    loop {
        match r.read(&mut buf).await {
            Ok(0) => {
                tracing::error!("inbound read EOF");
                break;
            }
            Ok(n) => {
                tracing::debug!("receive data size {}", n);
                // TODO: 分类数据
                let frame: super::ProxyFrame = bincode::deserialize(&buf[0..n])?;
                match frame {
                    super::ProxyFrame::Client(name, _local_port, remote_port) => {
                        tracing::debug!("receive proxy client {:?}", &name);
                        let addr = format!("0.0.0.0:{}", remote_port);
                        tracing::debug!("proxy server is listening at {}", &addr);
                        let listen = proxy_listen(name, addr, tx.clone(), brx.clone()).map(|r| {
                            if let Err(e) = r {
                                tracing::error!("proxy server bind error {}", e);
                            }
                        });
                        tokio::spawn(listen);
                    }
                    super::ProxyFrame::Body(name, addr, body) => {
                        tracing::debug!("name: {}", name);
                        btx.send((addr, body)).await?;
                    }
                }
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
        // tracing::debug!("GOT = {:?}", &message);
        w.write(&message).await?;
    }
    Ok(())
}

async fn proxy_listen(
    name: String,
    addr: String,
    tx: Sender<Vec<u8>>,
    brx: async_channel::Receiver<(String, Vec<u8>)>,
) -> Result<()> {
    tracing::debug!("proxy listener bind to {}", &addr);
    let listener = TcpListener::bind(addr).await?;
    while let Ok((mut inbound, peer_addr)) = listener.accept().await {
        tracing::info!("proxy stream addr {} is connected", &peer_addr);
        let (rin, win) = inbound.split();
        let peer = peer_addr.to_string();
        tokio::try_join!(
            proxy_listen_read(name.clone(), peer.clone(), rin, tx.clone()),
            proxy_listen_write(win, peer, brx.clone())
        )?;
    }
    Ok(())
}

async fn proxy_listen_read(
    name: String,
    peer_addr: String,
    mut inbound: ReadHalf<'_>,
    tx: Sender<Vec<u8>>,
) -> Result<()> {
    let mut buf = vec![0u8; 4096];
    loop {
        let name = name.clone();
        match inbound.read(&mut buf).await {
            Ok(0) => {
                tracing::error!("proxy stream disconnected");
                break;
            }
            Ok(n) => {
                tracing::debug!("proxy listen read size {}", n);
                let body = super::ProxyFrame::Body(name, peer_addr.clone(), buf[0..n].to_vec());
                let encoded: Vec<u8> = bincode::serialize(&body).unwrap();
                tx.send(encoded).await?;
            }
            Err(e) => {
                tracing::error!("proxy stream read error {}", e);
                break;
            }
        }
    }
    Ok(())
}

async fn proxy_listen_write(
    mut inbound: WriteHalf<'_>,
    peer_addr: String,
    brx: async_channel::Receiver<(String, Vec<u8>)>,
) -> Result<()> {
    while let Ok((addr, message)) = brx.recv().await {
        tracing::debug!("proxy listen write get");
        if peer_addr == addr {
            inbound.write(&message).await?;
        };
    }
    Ok(())
}
