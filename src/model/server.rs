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

use crate::model::config::ClientType;
use futures::{SinkExt, StreamExt};
use hyper::service::{make_service_fn, service_fn};
use hyper::upgrade::Upgraded;
use hyper::{client::connect::Connect, Body, Client, Request, Response, Server, StatusCode};
use hyper_tls::HttpsConnector;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::watch::{Receiver, Sender};
use tokio_tungstenite::{tungstenite::protocol, WebSocketStream};
use tracing::{debug, error, info};

pub struct Listener<T> {
    pub ws: WebSocketStream<T>,
}

pub struct AppServer {
    pub ws: WebSocketStream<Upgraded>,
    pub tx: Sender<()>,
    pub rx: Receiver<()>,
}

impl AppServer {
    pub fn new(ws: WebSocketStream<Upgraded>, tx: Sender<()>, rx: Receiver<()>) -> Self {
        Self { ws, tx, rx }
    }
    pub async fn ws_handle(&mut self) -> crate::Result<()> {
        // while let Some(msg) = self.ws.next().await {
        //     let msg = msg?;
        //     info!("websocket msg is \n{}", msg);
        //     let conf: std::result::Result<HashMap<String, crate::model::config::ClientConfig>, toml::de::Error> = toml::from_slice(&msg.into_data());
        //     match conf {
        //         Ok(c) => {
        //             info!("clients conf from ws");
        //             for i in c {
        //                 let rx = self.rx.clone();
        //                 match i.1.client_type {
        //                     ClientType::TCP => {
        //                         // tokio::task::spawn(client_tcp_handle(i.1.local_ip, i.1.local_port, i.1.remote_port, rx));
        //                     },
        //                     ClientType::UDP => {
        //                         // tokio::task::spawn(client_udp_handle(i.1.local_ip, i.1.local_port, i.1.remote_port, rx));
        //                     }
        //                     ClientType::HTTP => {
        //                         info!("app server http handle");
        //                         tokio::task::spawn(self.http_handle(i.1.local_ip, i.1.local_port, i.1.remote_port));
        //                     },
        //                     _ => {debug!("TODO")},
        //                 }

        //             }
        //         },
        //         Err(_e) => {self.ws.send(protocol::Message::binary("client config error")).await?;},
        //     };
        // };
        Ok(())
    }
    pub async fn http_handle(
        &mut self,
        local_ip: String,
        local_port: u16,
        remote_port: u16,
    ) -> crate::Result<()> {
        info!(
            "local http client {}:{}, remote port {}.",
            &local_ip, &local_port, &remote_port
        );
        // let addr = SocketAddr::from(([127, 0, 0, 1], remote_port));
        // let rx = self.rx.clone();
        // let new_service = make_service_fn(move |_| {
        //     let name = format!("http://{}:{}", local_ip, local_port);
        //     async { Ok::<_, hyper::Error>(service_fn(move |req| Self::handle_request(req, name.to_owned(), rx))) }
        // });
        // // 后续如果客户端断开，需要关闭服务端
        // let srv = Server::bind(&addr).serve(new_service).with_graceful_shutdown(async {
        //     self.rx.changed().await.unwrap();
        // });
        // srv.await?;
        Ok(())
    }
    // pub async fn handle_request(
    //     req: Request<Body>,
    //     name: String,
    //     mut rx: Receiver<()>,
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
    //             let e: crate::Error = e.into();
    //             Ok(e.into())
    //         }
    //     }
    // }
}
