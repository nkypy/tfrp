#![recursion_limit = "1024"]

#[macro_use]
extern crate log;

use clap::Clap;
use futures::future::{try_join_all,try_join};
use futures::join;
use futures::FutureExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server, StatusCode};
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use std::io::ErrorKind;
use std::net::SocketAddr;
use tokio::io;
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::stream::StreamExt;
use tokio::task;

use frp::error::Error;
use frp::Result;

#[derive(Clap)]
#[clap(name = "tfrps", version = "0.1.0", author = "Jack Shih")]
struct Opts {
    #[clap(short = "c", long = "config", default_value = "config/frps.toml")]
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

// async fn stream_data(
//     cryptor: frp::crypto::aead::AeadCryptor,
//     mut reader: ReadHalf,
//     mut writer: WriteHalf,
// ) -> () {
//     let mut buf = [0; 1024];
//     let mut written = 0;
//     loop {
//         let mut len = 0;
//         match reader.read(&mut buf).await {
//             Ok(0) => len = 0,
//             Ok(l) => len = l,
//             Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
//             Err(e) => return,
//         };
//         let mut enc_msg = vec![0u8; len + cryptor.tag_len];
//         cryptor.encrypt(&buf[..len], &mut enc_msg);
//         let mut dec_msg = vec![0u8; enc_msg.len() - cryptor.tag_len];
//         cryptor.decrypt(&enc_msg, &mut dec_msg);
//         writer.write_all(&dec_msg).await;
//         written += len as u64;
//     }
// }

// async fn handle_stream(cryptor: frp::crypto::aead::AeadCryptor, mut stream: TcpStream) -> Result<()> {
//     let mut conn = TcpStream::connect("10.9.1.127:8999").await?;
//     // let (mut reader, mut writer) = &mut (&stream, &stream);
//     // let (mut reader2, mut writer2) = &mut (&conn, &conn);
//     let (mut reader, mut writer) = stream.split();
//     let (mut reader2, mut writer2) = conn.split();
//     join!(
//         stream_data(cryptor.clone(), reader, writer2),
//         stream_data(cryptor.clone(), reader2, writer),
//     );
//     Ok(())
// }

#[cfg(target_arch = "aarch64")]
fn test_cfg() -> () {
    println!("this is aarch64");
}

#[cfg(not(target_arch = "aarch64"))]
fn test_cfg() -> () {
    println!("this is not aarch64");
}

async fn handle_request(
    req: Request<Body>,
    name: String,
) -> std::result::Result<Response<Body>, hyper::Error> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let url = req.uri().to_string();
    println!("url is {} method is {}", url, &req.method());
    let mut req = req;
    *req.uri_mut() = format!("{}{}", name, &url).parse::<hyper::Uri>().unwrap();
    // client.request(req).await
    let res = client.request(req).await;
    match res {
        Ok(body) => Ok(body),
        Err(e) => {
            error!("http client error {}", e);
            let e: Error = e.into();
            Ok(e.into())
        }
    }
}

async fn new_srv(name: String, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let new_service = make_service_fn(move |_| {
        let name = name.clone();
        async { Ok::<_, hyper::Error>(service_fn(move |req| handle_request(req, name.to_owned()))) }
    });
    let srv = Server::bind(&addr).serve(new_service);
    srv.await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Clap::parse();
    env_logger::init();
    let buf = std::fs::read_to_string(opts.config)?;
    let conf: Config = toml::from_str(&buf)?;
    let cryptor = frp::crypto::aead::AeadCryptor::new(conf.common.auth_token);
    let content =
        "content to encrypt, 哈哈，测试测试，试试 chacha20 加解密怎么样，下周得去上海了。"
            .as_bytes();
    let mut enc_msg = vec![0u8; content.len() + cryptor.tag_len];
    cryptor.encrypt(content, &mut enc_msg);
    let mut dec_msg = vec![0u8; enc_msg.len() - cryptor.tag_len];
    cryptor.decrypt(&enc_msg, &mut dec_msg);
    info!("decrypted data is {}", String::from_utf8(dec_msg.to_vec())?);
    test_cfg();
    let mut listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], conf.common.bind_port))).await?;
    let server = async move {
        let mut incoming = listener.incoming();
        while let Some(socket_res) = incoming.next().await {
            match socket_res {
                Ok(socket) => {
                    info!("accepted connection from {:?}", socket.peer_addr());
                    task::spawn(new_srv("http://10.16.2.74:8999".to_string(), 12303));
                    task::spawn(new_srv("https://10.16.8.108".to_string(), 12302));
                }
                Err(err) => {
                    error!("accept error = {:?}", err);
                }
            }
        }
    };
    info!("tfrp server is listening at 0.0.0.0:{}.", conf.common.bind_port);
    server.await;
    Ok(())
}
