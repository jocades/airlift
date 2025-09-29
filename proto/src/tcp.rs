use std::net::Ipv4Addr;

use anyhow::Result;
use axum::{
    Json, Router,
    extract::Path,
    routing::{get, post},
};
use tokio::net::TcpListener;
use tracing::info;
use uuid::Uuid;

use crate::{Info, Metadata, Offer, Peer};

pub async fn serve(port: u16) -> Result<()> {
    let router = Router::new()
        .route("/offer", post(offer))
        .route("/download/{id}", get(download));

    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await?;
    info!("Listening on port {port}...");
    axum::serve(listener, router).await?;
    Ok(())
}

async fn offer(Json(offer): Json<Offer>) {
    info!(files = ?offer.files, "Offer from {:?}", offer.from);

    // let mut offers = state.lock().await;
    // for file in offer.files {
    //     offers.insert(file.id, file);
    // }
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

    let resp = client.post(&url).json(&offer).send().await?;
    info!(status = ?resp.status(), "Sent offer to {}:{}", to.ip, to.info.port);

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
