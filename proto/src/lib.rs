use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

pub mod tcp;
pub mod udp;

pub type PeerId = Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Info {
    pub id: PeerId,
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

pub type PeerMap = Arc<Mutex<HashMap<PeerId, (Peer, Instant)>>>;

pub enum Event {
    Join(Peer),
    Leave(PeerId),
}
