use async_std::sync::Arc;
use async_std::task;
use clap::Parser;
use futures::join;
use std::time::Duration;

mod peer;
use peer::Peer;

/// A simple P2P CLI demo application
#[derive(Debug, Parser)]
#[command(name = "Peer", version, about)]
struct Args {
    /// Sets the messaging period
    #[arg(short = 'r', long, value_name = "SECONDS", default_value = "1")]
    period: u64,

    /// Sets the peer port
    #[arg(short, long)]
    port: u16,

    /// Sets the address of the remote peer
    #[arg(short, long, value_name = "HOST:PORT")]
    connect: Option<String>,
}

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let peer = Arc::new(Peer::new(args.port).await?);

    let run_task = {
        let peer = peer.clone();
        task::spawn(async move {
            peer.run().await.unwrap();
        })
    };

    let connect_task = {
        if let Some(addr) = args.connect {
            let peer = peer.clone();
            task::spawn(async move {
                peer.connect_to_peer(addr).await.unwrap();
            })
        } else {
            task::spawn(async {})
        }
    };

    let periodic_task = {
        let peer = peer.clone();
        task::spawn(async move {
            let interval = Duration::from_secs(args.period);
            loop {
                task::sleep(interval).await;
                peer.broadcast_message(b"Periodic message").await;
            }
        })
    };

    join!(run_task, connect_task, periodic_task);

    Ok(())
}
