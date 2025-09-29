use std::{collections::HashMap, path::Path, sync::Arc, time::Instant};

use serde::{Deserialize, Serialize};
use sysinfo::System;
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

pub enum Device {
    Desktop,
    Mobile,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            alias: "Peer".into(),
            port: 8000,
        }
    }
}

impl Info {
    pub fn new() {
        println!("System name:             {:?}", System::name());
        println!("System kernel version:   {:?}", System::kernel_version());
        println!("System OS version:       {:?}", System::os_version());
        println!("System host name:        {:?}", System::host_name());
    }

    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Info> {
        match Self::read(&path)? {
            Some(info) => Ok(info),
            None => {
                let info = Self::default();
                info.save(path)?;
                Ok(info)
            }
        }
    }

    pub fn read(path: impl AsRef<Path>) -> anyhow::Result<Option<Self>> {
        if !path.as_ref().exists() {
            return Ok(None);
        }
        Ok(Some(serde_json::from_slice(&std::fs::read(path)?)?))
    }

    pub fn save(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        std::fs::write(path, serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }
}
