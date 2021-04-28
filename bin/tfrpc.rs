/*
 * Copyright (c) 2020  [Jack Shih <i@kshih.com>]
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *    http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::collections::HashMap;
use std::convert::TryInto;
use std::fs;
use std::time::Duration;

use clap::Clap;
use futures::FutureExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use tfrp::conn::client;
use tfrp::protocol::ProxyFrame;
use tfrp::Result;

#[derive(Clap)]
#[clap(name = "tfrpc", version = tfrp::VERSION, author = tfrp::AUTHOR)]
struct Opts {
    #[clap(short = 'c', long = "config", default_value = "config/tfrpc.toml")]
    config: String,
}

#[derive(Debug)]
pub struct Listener {
    pub conn: TcpStream,
    pub proxy: HashMap<String, TcpStream>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Clap::parse();
    tracing_subscriber::fmt().init();
    let conf: tfrp::ClientConfig = toml::from_str(&fs::read_to_string(opts.config)?)?;
    let addr = format!("{}:{}", conf.common.server_addr, conf.common.server_port);
    tracing::info!("tfrp client is connecting to {}", &addr);
    dbg!(&conf.clients);
    let mut stream = TcpStream::connect(addr).await?;
    tracing::info!(
        "client stream locol addr is {}",
        &stream.local_addr().unwrap()
    );
    // let (r, w) = stream.split();
    // let (tx, rx) = mpsc::channel(10);
    // let (btx, brx) = broadcast::channel(10);
    // let mut jobs = vec![];
    // jobs.push(client::handle_read(r, btx.clone()).boxed());
    // jobs.push(client::handle_write(w, rx).boxed());
    for (k, v) in conf.clients {
        let local_port = v.local_port.clone();
        tracing::info!("write {}", &local_port);
        stream
            .write(&bincode::serialize(&ProxyFrame::Conf(k.clone(), v)).unwrap())
            .await;
        // let tx2 = tx.clone();

        // tx2.send(ProxyFrame::Conf(k.clone(), v)).await?;
        // let brx2 = btx.subscribe();
        // if let Ok(mut conn) = TcpStream::connect(format!("127.0.0.1:{}", local_port)).await {
        //     tracing::debug!("proxy stream locol addr is {}", conn.local_addr().unwrap());
        //     jobs.push(client::handle_transfer(conn, tx2, brx2).boxed());
        // };
    }
    let peer_addr = stream.local_addr().unwrap().to_string();
    let mut buf = vec![0u8; 4096];
    loop {
        match stream.read(&mut buf).await {
            Ok(0) => {
                tracing::error!("inbound from {} read EOF", peer_addr);
                break;
            }
            Ok(n) => match buf[0..n].to_vec().try_into() {
                Ok(ProxyFrame::Body(name, addr, body)) => {}
                _ => {
                    tracing::error!("inbound from {} data serialize failed", peer_addr);
                }
            },
            Err(e) => {
                tracing::error!("inbound from {} read error {}", peer_addr, e);
                break;
            }
        }
    }
    // futures::future::try_join_all(jobs).await?;
    Ok(())
}
