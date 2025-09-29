use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use axum::Json;
use axum::response::IntoResponse;
use axum::{
    Router,
    extract::{Path, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::Mutex;
use tracing::{debug, info, trace};
use uuid::Uuid;

pub mod udp;

pub const UDP_HOST: &str = "224.0.0.167";
pub const UDP_PORT: u16 = 53317;
pub const UDP_ADDR: (&str, u16) = (UDP_HOST, UDP_PORT);
pub const TCP_PORT: u16 = 8000;

#[derive(Clone)]
struct AppState {
    info: Arc<Info>,
    peers: Arc<Mutex<Vec<Peer>>>,
    offers: Arc<Mutex<HashMap<Uuid, Metadata>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Info {
    pub id: Uuid,
    pub alias: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct Peer {
    pub info: Info,
    pub ip: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub id: Uuid,
    pub filename: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Offer {
    from: Info,
    files: Vec<Metadata>,
}

pub async fn serve(info: Arc<Info>, peers: Arc<Mutex<Vec<Peer>>>) -> Result<()> {
    let port = info.port;

    let state = AppState {
        info,
        peers,
        offers: Arc::new(Mutex::new(HashMap::new())),
    };

    let router = Router::new()
        .route("/offer", post(offer))
        .route("/download/{id}", get(download))
        .with_state(state);

    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await?;
    info!("Listening on port {port}...");
    axum::serve(listener, router).await?;
    Ok(())
}

async fn offer(State(state): State<AppState>, Json(offer): Json<Offer>) {
    info!(files = ?offer.files, "Offer from {:?}", offer.from);

    let mut offers = state.offers.lock().await;
    for file in offer.files {
        offers.insert(file.id, file);
    }
}

async fn download(Path(id): Path<Uuid>) {
    info!(?id, "Download");
}

pub async fn offer_file(info: &Info, path: impl AsRef<std::path::Path>, to: &Peer) -> Result<()> {
    let size = tokio::fs::metadata(&path).await?.len();
    let meta = Metadata {
        id: Uuid::new_v4(),
        filename: path.as_ref().file_name().unwrap().to_string_lossy().into(),
        size,
    };

    let offer = Offer {
        from: info.clone(),
        files: vec![meta],
    };

    let client = reqwest::Client::new();
    let url = format!("http://{}:{}/offer", to.ip, to.info.port);

    client.post(&url).json(&offer).send().await?;

    Ok(())
}

async fn download_file(ip: &str, port: u16, id: Uuid, filename: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("http://{}:{}/download/{}", ip, port, id);
    let data = client.get(&url).send().await?.bytes().await?;
    tokio::fs::write(format!("data/down/{filename}"), &data).await?;
    info!("Downloaded {filename}");
    Ok(())
}

pub async fn listen(
    id: Uuid,
    peers: Arc<Mutex<Vec<Peer>>>,
    tx: tokio::sync::mpsc::Sender<Peer>,
) -> Result<()> {
    let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, UDP_PORT)).await?;
    sock.join_multicast_v4(UDP_HOST.parse::<Ipv4Addr>()?, Ipv4Addr::UNSPECIFIED)?;

    info!("Listening for multicast on {UDP_HOST}:{UDP_PORT}...");

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
        if !peers
            .iter()
            .any(|p| p.info.id == info.id && p.info.port == info.port)
        {
            info!("Discovered new peer {info:?} at {}", addr.ip());
            let peer = Peer {
                info,
                ip: addr.ip().to_string(),
            };

            peers.push(peer.clone());
            tx.send(peer).await?;
        }
    }
}

pub async fn announce(info: Arc<Info>) -> Result<()> {
    let message = serde_json::to_vec(info.as_ref())?;

    let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
    let target = UDP_ADDR;

    info!("Anouncing to {target:?} every 2s...");
    loop {
        sock.send_to(&message, target).await?;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}
