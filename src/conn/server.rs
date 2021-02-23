use std::convert::TryInto;

use futures::FutureExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{
    tcp::{ReadHalf, WriteHalf},
    TcpListener, TcpStream,
};
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
    let (tx, rx) = mpsc::channel::<(bool, Vec<u8>)>(32);
    tokio::try_join!(transfer_stream_read(r, tx), transfer_stream_write(w, rx))?;
    Ok(())
}

async fn transfer_stream_read(mut r: ReadHalf<'_>, mut tx: Sender<(bool, Vec<u8>)>) -> Result<()> {
    let mut buf = vec![0u8; 4096];
    let (mut btx, mut brx) = async_channel::unbounded();
    loop {
        match r.read(&mut buf).await {
            Ok(0) => {
                tracing::error!("inbound read EOF");
                tx.send((false, vec![])).await?;
                break;
            }
            Ok(n) => {
                tracing::debug!("receive data size {}", n);
                // TODO: 分类数据
                if let Ok(frame) = buf[0..n].to_vec().try_into() {
                    match frame {
                        crate::protocol::ProxyFrame::Conf(name, _local_port, remote_port) => {
                            tracing::debug!("receive proxy client {:?}", &name);
                            let addr = format!("0.0.0.0:{}", remote_port);
                            tracing::debug!("proxy server is listening at {}", &addr);
                            let listen =
                                proxy_listen(name, addr, tx.clone(), brx.clone()).map(|r| {
                                    if let Err(e) = r {
                                        tracing::error!("proxy server bind error {}", e);
                                    }
                                });
                            tokio::spawn(listen);
                        }
                        crate::protocol::ProxyFrame::Body(name, addr, body) => {
                            tracing::debug!("name: {}", name);
                            btx.send((addr, body)).await?;
                        }
                    }
                } else {
                    tx.send((false, vec![])).await?;
                };
            }
            Err(e) => {
                tracing::error!("inbound read error {}", e);
                break;
            }
        }
    }
    Ok(())
}

async fn transfer_stream_write(
    mut w: WriteHalf<'_>,
    mut rx: Receiver<(bool, Vec<u8>)>,
) -> Result<()> {
    while let Some(message) = rx.recv().await {
        // tracing::debug!("GOT = {:?}", &message);
        if message.0 != true {
            tracing::debug!("stream write not found page");
            w.write(crate::error::HTTP_NOT_FOUND_HTML.as_bytes())
                .await?;
        } else {
            tracing::debug!("stream write data");
            w.write(&message.1).await?;
        }
    }
    Ok(())
}

async fn proxy_listen(
    name: String,
    addr: String,
    tx: Sender<(bool, Vec<u8>)>,
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
    tx: Sender<(bool, Vec<u8>)>,
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
                let body =
                    crate::protocol::ProxyFrame::Body(name, peer_addr.clone(), buf[0..n].to_vec());
                let encoded: Vec<u8> = bincode::serialize(&body).unwrap();
                tx.send((true, encoded)).await?;
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
