use std::{collections::HashMap, sync::Arc, time::Duration};

use serde_json::json;
use tauri::{async_runtime::spawn, AppHandle, Emitter, Manager};
use tokio::{
    sync::{mpsc, Mutex},
    time,
};
use tracing::{debug, error, Level};

use proto::{
    tcp::serve,
    udp::{announce, listen},
    Event, Info, PeerMap,
};

#[derive(Clone)]
struct AppState {
    info: Arc<Info>,
    peers: PeerMap,
}

pub async fn emiter(handle: AppHandle, mut rx: mpsc::Receiver<Event>) -> anyhow::Result<()> {
    while let Some(event) = rx.recv().await {
        match event {
            Event::Join(peer) => {
                handle.emit("events", json!({"kind": "join", "peer": peer}))?;
            }
            Event::Leave(id) => {
                handle.emit("events", json!({"kind": "leave", "id": id}))?;
            }
        };
    }
    Ok(())
}

pub async fn cleanup(peers: PeerMap, tx: mpsc::Sender<Event>) {
    let timeout = Duration::from_secs(10);
    let mut removed = Vec::new();

    loop {
        let mut peers = peers.lock().await;
        peers.retain(|_, (peer, last_seen)| {
            let timedout = last_seen.elapsed() > timeout;
            if timedout {
                removed.push(peer.info.id);
            }
            !timedout
        });
        drop(peers);

        for id in &removed {
            debug!("Removing {id}");
            _ = tx.send(Event::Leave(*id)).await;
        }
        removed.clear();
        time::sleep(Duration::from_secs(5)).await;
    }
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState {
        info: Arc::new(Info::from_path("device.json")?),
        peers: Arc::new(Mutex::new(HashMap::new())),
    };

    let (tx, rx) = mpsc::channel(32);
    {
        let (id, peers, tx) = (state.info.id, state.peers.clone(), tx.clone());
        let callback = async move |peer| _ = tx.send(Event::Join(peer)).await;
        spawn(async move {
            if let Err(e) = listen(id, peers, callback).await {
                error!(cause = %e, "listen error");
            }
        });
    }

    {
        let info = state.info.clone();
        spawn(async move {
            if let Err(e) = announce(info).await {
                error!(cause = %e, "announce error");
            }
        });
    }

    {
        let handle = app.handle().clone();
        spawn(async move {
            if let Err(e) = emiter(handle, rx).await {
                error!(cause = %e, "emiter error");
            }
        });
    }

    {
        let peers = state.peers.clone();
        spawn(cleanup(peers, tx));
    }

    {
        let port = state.info.port;
        spawn(async move {
            if let Err(e) = serve(port).await {
                error!(cause = %e, "serve error");
            }
        });
    }

    app.manage(state);

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![])
        .setup(setup)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
