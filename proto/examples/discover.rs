use std::net::Ipv4Addr;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tokio::net::UdpSocket;

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

pub const HOST: &str = "224.0.0.167";
pub const PORT: u16 = 53317;

async fn listen() -> Result<()> {
    let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, PORT)).await?;
    sock.join_multicast_v4(HOST.parse::<Ipv4Addr>()?, Ipv4Addr::UNSPECIFIED)?;

    let mut buf = [0; 1024];
    loop {
        let (n, addr) = sock.recv_from(&mut buf).await?;
        if n == 0 {
            continue;
        }
        println!(
            "Received from {}: {}",
            addr.ip(),
            String::from_utf8_lossy(&buf[..n])
        );
    }
}

async fn announce() -> Result<()> {
    let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
    let target = (HOST, PORT);
    loop {
        sock.send_to("hello".as_bytes(), target).await?;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
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
