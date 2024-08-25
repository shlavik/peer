use async_std::net::SocketAddr;
use async_std::task;
use clap::Parser;
use futures::join;

use peer::{DirectFlood, GossipDiscovery, Peer, PeerStore};

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
    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    let peer_store = PeerStore::new();
    let peer = Peer::new(addr, peer_store.clone()).await?;
    let discovery = GossipDiscovery::new(peer.clone(), peer_store.clone(), args.period);
    let flood = DirectFlood::new(peer.clone(), peer_store, args.period);

    peer.clone().register_protocol(discovery.clone()).await?;
    peer.clone().register_protocol(flood.clone()).await?;

    let peer_task = {
        let peer = peer.clone();
        task::spawn(async move {
            peer.start().await.unwrap();
        })
    };

    let discovery_task = {
        task::spawn(async move {
            discovery.start().await.unwrap();
        })
    };

    let flood_task = {
        task::spawn(async move {
            flood.start().await.unwrap();
        })
    };

    let connect_task = {
        if let Some(addr_str) = args.connect {
            let addr = addr_str.parse::<SocketAddr>().unwrap();
            task::spawn(async move {
                peer.connect_to_peer(addr).await.unwrap();
            })
        } else {
            task::spawn(async {})
        }
    };

    join!(peer_task, discovery_task, flood_task, connect_task);

    Ok(())
}
