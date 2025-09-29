use std::net::Ipv4Addr;

use anyhow::Result;
use tokio::net::UdpSocket;

pub const HOST: &str = "224.0.0.167";
pub const PORT: u16 = 53317;

pub async fn listen() -> Result<()> {
    let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, PORT)).await?;
    sock.join_multicast_v4(HOST.parse::<Ipv4Addr>()?, Ipv4Addr::UNSPECIFIED)?;

    let mut buf = [0; 1024];
    loop {
        let (n, addr) = sock.recv_from(&mut buf).await?;
        if n == 0 {
            continue;
        }
        println!(
            "Received from {}: {}",
            addr.ip(),
            String::from_utf8_lossy(&buf[..n])
        );
    }
}

pub async fn announce() -> Result<()> {
    let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
    let target = (HOST, PORT);
    loop {
        sock.send_to("hello".as_bytes(), target).await?;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}
