use async_std::io::{ReadExt, WriteExt};
use async_std::net::{SocketAddr, TcpListener, TcpStream};
use async_std::sync::{Arc, Mutex};
use async_std::task;
use std::collections::HashMap;
use std::io::Result;

use crate::message::Message;

#[async_trait::async_trait]
pub trait Protocol {
    async fn handle_message(&self, peer: Arc<Peer>, addr: SocketAddr, message: Message);
}

pub struct Peer {
    pub listener: TcpListener,
    connections: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
    protocol: Arc<dyn Protocol + Send + Sync>,
}

impl Peer {
    pub async fn new(port: u16, protocol: Arc<dyn Protocol + Send + Sync>) -> Result<Self> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(addr).await?;
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
        let mut buffer = [0; 1024];
        while let Ok(n) = stream.read(&mut buffer).await {
            if n == 0 {
                break;
            }
            self.protocol
                .handle_message(self.clone(), addr, buffer.into())
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

    pub async fn broadcast_message(
        &self,
        message: Message,
        except_addr: Option<SocketAddr>,
    ) -> Result<()> {
        let buffer: [u8; 1024] = message.into();
        for (addr, mut stream) in &*self.connections.lock().await {
            if let Some(except_addr) = except_addr {
                if *addr == except_addr {
                    continue;
                }
            }
            if let Err(e) = stream.write_all(&buffer).await {
                eprintln!("Failed to send message to {}: {}", addr, e);
            }
        }
        Ok(())
    }

    pub async fn connect_to_peer(self: Arc<Self>, addr: SocketAddr) -> Result<()> {
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
