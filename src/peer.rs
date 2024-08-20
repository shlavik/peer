use async_std::io::{ReadExt, WriteExt};
use async_std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use async_std::sync::{Arc, Mutex};
use async_std::task;
use std::collections::HashMap;
use std::io::Result;

#[async_trait::async_trait]
pub trait Protocol {
    async fn handle_message(&self, peer: Arc<Peer>, addr: SocketAddr, message: Vec<u8>);
}

pub struct GossipProtocol;

#[async_trait::async_trait]
impl Protocol for GossipProtocol {
    async fn handle_message(&self, peer: Arc<Peer>, addr: SocketAddr, message: Vec<u8>) {
        println!(
            "Gossip received from {}: {}",
            addr,
            String::from_utf8_lossy(&message)
        );
        peer.broadcast_message(&message, Some(addr)).await;
    }
}

pub struct Peer {
    listener: TcpListener,
    connections: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
    protocol: Arc<dyn Protocol + Send + Sync>,
}

impl Peer {
    pub async fn new(port: u16, protocol: Arc<dyn Protocol + Send + Sync>) -> Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", port)).await?;
        Ok(Self {
            listener,
            connections: Arc::new(Mutex::new(HashMap::new())),
            protocol,
        })
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        println!("Listening on {}", self.listener.local_addr()?);
        while let Ok((stream, _)) = self.listener.accept().await {
            let peer = self.clone();
            task::spawn(async move {
                if let Err(e) = peer.handle_connection(stream).await {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }
        Ok(())
    }

    async fn handle_connection(self: Arc<Self>, mut stream: TcpStream) -> Result<()> {
        let addr = stream.peer_addr()?;
        self.connections.lock().await.insert(addr, stream.clone());
        let mut buffer = vec![0u8; 1024];
        while let Ok(n) = stream.read(&mut buffer).await {
            if n == 0 {
                break;
            }
            self.protocol
                .handle_message(self.clone(), addr, buffer[..n].to_vec())
                .await;
        }
        self.remove_peer(addr).await;
        Ok(())
    }

    async fn remove_peer(self: Arc<Self>, addr: SocketAddr) {
        let mut connections = self.connections.lock().await;
        connections.remove(&addr);
        println!(
            "Peer {} removed. Active peers: {:?}",
            addr,
            connections.keys()
        );
    }

    pub async fn broadcast_message(&self, message: &[u8], except_addr: Option<SocketAddr>) {
        let connections = self.connections.lock().await;
        for (addr, mut stream) in &*connections {
            if let Some(except_addr) = except_addr {
                if *addr == except_addr {
                    continue;
                }
            }
            if let Err(e) = stream.write_all(message).await {
                eprintln!("Failed to send message to {}: {}", addr, e);
            }
        }
    }

    pub async fn connect_to_peer(self: Arc<Self>, addr: impl ToSocketAddrs) -> Result<()> {
        let stream = TcpStream::connect(addr).await?;
        let peer = self.clone();
        task::spawn(async move {
            if let Err(e) = peer.handle_connection(stream).await {
                eprintln!("Error in connection: {}", e);
            }
        });
        Ok(())
    }
}
