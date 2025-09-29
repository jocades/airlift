use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand};
use proto::udp::announce;
use tracing::Level;
use uuid::Uuid;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = 8000)]
    port: u16,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Listen,
    Announce {
        #[arg(long, default_value = "Peer")]
        alias: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let args = Args::parse();

    let mut info = proto::Info {
        id: Uuid::new_v4(),
        alias: "Peer".into(),
        port: args.port,
    };

    // let peers = Arc::new(Mutex::new(Vec::new()));

    match args.command {
        Command::Listen => {
            // proto::listen(peers).await?;
        }
        Command::Announce { alias } => {
            info.alias = alias;
            announce(Arc::new(info)).await?;
        }
    }

    Ok(())
}
