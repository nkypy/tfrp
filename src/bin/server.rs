#![recursion_limit = "1024"]

#[macro_use]
extern crate log;

use async_std::io;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use clap::Clap;
use futures::join;
use serde::Deserialize;
use std::io::ErrorKind;

use frp::Result;

#[derive(Clap)]
#[clap(name = "frps", version = "0.1.0", author = "Jack Shih")]
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

async fn stream_data(
    cryptor: frp::crypto::aead::AeadCryptor,
    mut reader: &TcpStream,
    mut writer: &TcpStream,
) -> () {
    let mut buf = [0; 1024];
    let mut written = 0;
    loop {
        let mut len = 0;
        match reader.read(&mut buf).await {
            Ok(0) => len = 0,
            Ok(l) => len = l,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return,
        };
        let mut enc_msg = vec![0u8; len + cryptor.tag_len];
        cryptor.encrypt(&buf[..len], &mut enc_msg);
        let mut dec_msg = vec![0u8; enc_msg.len() - cryptor.tag_len];
        cryptor.decrypt(&enc_msg, &mut dec_msg);
        writer.write_all(&dec_msg).await;
        written += len as u64;
    }
}

async fn handle_stream(cryptor: frp::crypto::aead::AeadCryptor, stream: TcpStream) -> Result<()> {
    let conn = TcpStream::connect("10.9.1.127:8999").await?;
    let (mut reader, mut writer) = &mut (&stream, &stream);
    let (mut reader2, mut writer2) = &mut (&conn, &conn);
    join!(
        stream_data(cryptor.clone(), reader, writer2),
        stream_data(cryptor.clone(), reader2, writer),
    );
    Ok(())
}

#[async_std::main]
async fn main() -> Result<()> {
    let opts: Opts = Clap::parse();
    env_logger::init();
    info!("this is server");
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
    let listener = TcpListener::bind(format!("{}:{}", "127.0.0.1", conf.common.bind_port)).await?;
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        task::spawn(handle_stream(cryptor.clone(), stream));
    }
    Ok(())
}
