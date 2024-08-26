use anyhow::Result;
use async_std::io::WriteExt;
use async_std::sync::Arc;
use async_std::task;
use std::time::Duration;

use crate::*;

pub struct GossipDiscovery {
    peer: Arc<Peer>,
    peer_store: Arc<PeerStore>,
    period: u64,
}

impl GossipDiscovery {
    pub fn new(peer: Arc<Peer>, peer_store: Arc<PeerStore>, period: u64) -> Arc<Self> {
        Arc::new(GossipDiscovery {
            peer,
            peer_store,
            period,
        })
    }

    pub async fn start(self: Arc<Self>) -> Result<()> {
        let message = Message::new(2, self.peer.get_addr(), None);
        let this = self.clone();
        task::spawn(async move {
            let interval = Duration::from_secs(this.period);
            loop {
                task::sleep(interval).await;
                if let Err(e) = this.broadcast_message(message.clone()).await {
                    eprintln!("Failed to broadcast message: {e:?}");
                }
            }
        });
        Ok(())
    }

    pub async fn broadcast_message(&self, message: Message) -> Result<()> {
        let buffer: [u8; Peer::BUFFER_SIZE] = MessageKind::Broadcast(message.clone()).into();
        for (addr, mut stream) in self.peer_store.get_peers().await {
            if addr != message.from {
                if let Err(e) = stream.write_all(&buffer).await {
                    let _ = self.peer_store.clone().remove_peer(addr).await;
                    eprintln!("Failed to broadcast message: {e:?}");
                }
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl Protocol for GossipDiscovery {
    async fn handle_message(&self, kind: &MessageKind) {
        if let MessageKind::Broadcast(message) = kind {
            if message.ttl > 0 && message.from != self.peer.clone().get_addr() {
                let _ = self.peer.clone().connect_to_peer(message.from).await;
                let message = Message::new(message.ttl - 1, message.from, None);
                if let Err(e) = self.broadcast_message(message).await {
                    eprintln!("Failed to broadcast message: {e:?}");
                }
            }
        }
    }
}
