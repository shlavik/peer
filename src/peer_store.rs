use anyhow::Result;
use async_std::net::{SocketAddr, TcpStream};
use async_std::sync::{Arc, Mutex};
use futures::AsyncWriteExt;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct PeerStore {
    peers: Mutex<HashMap<SocketAddr, TcpStream>>,
}

impl PeerStore {
    pub fn new() -> Arc<Self> {
        let peers = Mutex::new(HashMap::new());
        Arc::new(PeerStore { peers })
    }

    pub async fn get_peers(&self) -> HashMap<SocketAddr, TcpStream> {
        self.peers.lock().await.clone()
    }

    pub async fn has_peer(self: Arc<Self>, addr: &SocketAddr) -> bool {
        self.peers.lock().await.contains_key(addr)
    }

    pub async fn add_peer(self: Arc<Self>, addr: SocketAddr, stream: TcpStream) -> Result<()> {
        self.peers.lock().await.insert(addr, stream);
        Ok(())
    }

    pub async fn remove_peer(self: Arc<Self>, addr: SocketAddr) -> Result<()> {
        if let Some(mut stream) = self.peers.lock().await.remove(&addr) {
            if let Err(e) = stream.close().await {
                eprintln!("Failed to close stream: {e:?}");
            }
        }
        Ok(())
    }
}
