#[derive(Debug)]
pub enum Protocol {
    HTTP,
    HTTPS,
    KCP,
    TCP,
    UDP,
    WS,
    WSS,
}
