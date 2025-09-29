use anyhow::Result;
use clap::{Parser, Subcommand};
use proto::udp::{announce, listen};

/*
 t1: cargo r --example discover -- listen
 t2: cargo r --example discover -- announce
*/

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Listen,
    Announce,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Listen => listen().await?,
        Command::Announce => announce().await?,
    }

    Ok(())
}
