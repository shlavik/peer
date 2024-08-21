use async_std::net::SocketAddr;
use async_std::sync::Arc;

use crate::message::Message;
use crate::peer::{Peer, Protocol};

pub struct GossipProtocol;

#[async_trait::async_trait]
impl Protocol for GossipProtocol {
    async fn handle_message(&self, peer: Arc<Peer>, addr: SocketAddr, message: Message) {
        println!("Gossip from {}: {}", addr, message);
        let _ = peer.broadcast_message(message.clone(), Some(addr)).await;
        // let _ = peer.connect_to_peer(message.source).await;
    }
}
