use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::Level;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = proto::TCP_PORT)]
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

    /* match args.command {
        Command::Listen => {
            proto::listen().await?;
        }
        Command::Announce { alias } => {
            proto::announce(alias, 8000).await?;
        }
    } */

    Ok(())
}
