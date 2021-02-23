use async_net::TcpStream;

#[async_trait::async_trait]
impl super::ProxyClient for TcpStream {
    async fn name() -> String {
        "client".to_string()
    }
    async fn read() -> crate::Result<()> {
        Ok(())
    }
    async fn write() -> crate::Result<()> {
        Ok(())
    }
    async fn close() {}
}
