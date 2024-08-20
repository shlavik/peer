use async_std::io::{ReadExt, WriteExt};
use async_std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use async_std::sync::{Arc, Mutex};
use async_std::task;
use std::collections::HashMap;
use std::io::Result;

pub struct Peer {
    listener: TcpListener,
    connections: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
}

impl Peer {
    pub async fn new(port: u16) -> Result<Self> {
        let listener = TcpListener::bind(("0.0.0.0", port)).await?;
        Ok(Self {
            listener,
            connections: Arc::new(Mutex::new(HashMap::new())),
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
            println!(
                "Received from {}: {}",
                addr,
                String::from_utf8_lossy(&buffer[..n])
            );
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

    pub async fn broadcast_message(&self, message: &[u8]) {
        let connections = self.connections.lock().await;
        for mut stream in connections.values() {
            if let Err(e) = stream.write_all(message).await {
                eprintln!("Failed to send message: {}", e);
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
