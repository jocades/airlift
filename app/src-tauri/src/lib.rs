use proto::{listen, serve, Info, Metadata, Peer};
use std::{collections::HashMap, sync::Arc};
use tauri::{
    async_runtime::{channel, spawn},
    Emitter, Manager,
};
use tokio::sync::Mutex;
use tracing::{error, Level};
use uuid::Uuid;

const TCP_PORT: u16 = 8001;

#[derive(Clone)]
struct AppState {
    info: Arc<Info>,
    peers: Arc<Mutex<Vec<Peer>>>,
    // offers: Arc<Mutex<HashMap<Uuid, Metadata>>>,
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let id = Uuid::new_v4();
    let peers = Arc::new(Mutex::new(Vec::new()));

    let (tx, mut rx) = channel(32);
    {
        let peers = peers.clone();
        spawn(async move {
            if let Err(e) = listen(id, peers, tx).await {
                error!(cause = %e, "announce error");
            }
        });
    }

    {
        let handle = app.handle().clone();
        spawn(async move {
            while let Some(peer) = rx.recv().await {
                handle.emit("peer-joined", peer).unwrap();
            }
        });
    }

    let info = Info {
        id,
        alias: "Peer".into(),
        port: 8000,
    };

    let state = AppState {
        info: Arc::new(info),
        peers,
    };

    {
        let info = state.info.clone();
        let peers = state.peers.clone();
        spawn(async {
            if let Err(e) = proto::serve(info, peers).await {
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
