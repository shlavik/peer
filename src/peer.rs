use async_std::io::{ReadExt, WriteExt};
use async_std::net::{SocketAddr, TcpListener, TcpStream};
use async_std::sync::{Arc, Mutex};
use async_std::task;
use std::io::Result;

use crate::*;

pub struct Peer {
    addr: SocketAddr,
    listener: TcpListener,
    protocols: Arc<Mutex<Vec<Arc<dyn Protocol>>>>,
    peer_store: Arc<PeerStore>,
}

impl Peer {
    pub const BUFFER_SIZE: usize = 1024;

    pub async fn new(addr: SocketAddr, peer_store: Arc<PeerStore>) -> Result<Arc<Self>> {
        Ok(Arc::new(Peer {
            addr,
            listener: TcpListener::bind(addr).await?,
            protocols: Arc::new(Mutex::new(Vec::new())),
            peer_store,
        }))
    }

    pub fn get_addr(&self) -> SocketAddr {
        self.addr
    }

    pub async fn register_protocol(&self, protocol: Arc<dyn Protocol>) -> Result<()> {
        self.protocols.lock().await.push(protocol);
        Ok(())
    }

    pub async fn add_peer(&self, addr: SocketAddr, stream: TcpStream) -> Result<()> {
        self.peer_store.clone().add_peer(addr, stream).await?;
        let peer_addrs = self
            .peer_store
            .get_peers()
            .await
            .keys()
            .map(|addr| addr.to_string())
            .collect::<Vec<_>>()
            .join("\", \"");
        log(format!("Connected to the peers at [\"{}\"]", peer_addrs));
        Ok(())
    }

    pub async fn start(self: Arc<Self>) -> Result<()> {
        log(format!("My address is \"{}\"", self.listener.local_addr()?));
        while let Ok((stream, _)) = self.listener.accept().await {
            let this = self.clone();
            task::spawn(async move {
                if let Err(e) = this.handle_connection(stream).await {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }
        Ok(())
    }

    async fn handle_connection(&self, mut stream: TcpStream) -> Result<()> {
        let mut buffer = [0; Self::BUFFER_SIZE];
        while let Ok(n) = stream.read(&mut buffer).await {
            if n == 0 {
                break;
            };
            let kind = MessageKind::from(buffer);
            if let MessageKind::Handshake(message) = &kind {
                let _ = self.add_peer(message.from, stream.clone()).await;
            }
            for protocol in self.protocols.lock().await.iter() {
                protocol.handle_message(&kind).await;
            }
        }
        Ok(())
    }

    pub async fn connect_to_peer(self: Arc<Self>, addr: SocketAddr) -> Result<()> {
        if addr == self.get_addr() {
            return Ok(());
        }
        if self.peer_store.clone().has_peer(&addr).await {
            return Ok(());
        }
        let stream = TcpStream::connect(addr).await?;
        let _ = self.add_peer(addr, stream.clone()).await;
        let this = self.clone();
        task::spawn(async move {
            if let Err(e) = this.shake_hand().await {
                eprintln!("Error shaking hand: {}", e);
            }
        });
        let this = self.clone();
        task::spawn(async move {
            if let Err(e) = this.handle_connection(stream).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
        Ok(())
    }

    pub async fn shake_hand(&self) -> Result<()> {
        let message = MessageKind::Handshake(Message::new(0, self.get_addr(), None));
        let buffer: [u8; Self::BUFFER_SIZE] = message.into();
        for (_, mut stream) in self.peer_store.get_peers().await {
            if let Err(e) = stream.write_all(&buffer).await {
                eprintln!("Failed to send handshake: {}", e);
            }
        }
        Ok(())
    }
}
