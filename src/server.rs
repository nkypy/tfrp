use pin_project::pin_project;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

#[derive(Debug, Default)]
#[pin_project]
pub struct Server {
    #[pin]
    pub addr: String,
    pub client: Arc<Vec<TcpStream>>,
}

impl Server {
    pub fn new() -> Self {
        Self::default()
    }
    pub async fn listen<A: ToSocketAddrs>(&mut self, addr: A) -> crate::Result<()> {
        // tracing::info!("tfrps bind to {}", &addr.to_string());
        tracing::debug!("tfrps listen");
        let listener = TcpListener::bind(addr).await?;
        while let Ok((inbound, peer_addr)) = listener.accept().await {
            // tracing::debug!("peer addr is {}", peer_addr.to_string());
            dbg!(peer_addr);
            self.with(inbound);
        }
        Ok(())
    }
    pub fn with(&mut self, inbound: TcpStream) -> &mut Self {
        let c = Arc::get_mut(&mut self.client).unwrap();
        c.push(inbound);
        dbg!(self)
    }
}
