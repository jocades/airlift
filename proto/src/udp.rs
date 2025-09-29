use std::{net::Ipv4Addr, sync::Arc, time::Instant};

use anyhow::Result;
use tokio::{net::UdpSocket, time};
use tracing::info;

use crate::{Info, Peer, PeerId, PeerMap};

pub const HOST: &str = "224.0.0.167";
pub const PORT: u16 = 53317;

pub async fn listen(id: PeerId, peers: PeerMap, cb: impl AsyncFn(Peer)) -> Result<()> {
    let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, PORT)).await?;
    sock.join_multicast_v4(HOST.parse::<Ipv4Addr>()?, Ipv4Addr::UNSPECIFIED)?;

    info!("Listening for multicast on {HOST}:{PORT}...");

    let mut buf = [0; 1024];
    loop {
        let (n, addr) = sock.recv_from(&mut buf).await?;
        if n == 0 {
            continue;
        }

        let Ok(info) = serde_json::from_slice::<Info>(&buf[..n]) else {
            info!(
                "Unknown message from {addr}: {}",
                String::from_utf8_lossy(&buf[..n])
            );
            continue;
        };

        if info.id == id {
            continue;
        }

        let mut peers = peers.lock().await;

        if let Some((_, last_seen)) = peers.get_mut(&info.id) {
            *last_seen = Instant::now();
        } else {
            let ip = addr.ip();
            info!("Discovered new peer {info:?} at {ip}");

            let peer = Peer {
                info,
                ip: ip.to_string(),
            };

            peers.insert(peer.info.id, (peer.clone(), Instant::now()));
            cb(peer).await;
        }
    }
}

pub async fn announce(info: Arc<Info>) -> Result<()> {
    let message = serde_json::to_vec(info.as_ref())?;

    let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
    let target = (HOST, PORT);

    info!("Anouncing to {target:?} every 2s...");
    loop {
        sock.send_to(&message, target).await?;
        time::sleep(std::time::Duration::from_secs(2)).await;
    }
}
