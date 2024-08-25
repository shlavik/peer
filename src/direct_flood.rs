use async_std::io::WriteExt;
use async_std::sync::Arc;
use async_std::task;
use std::io::Result;
use std::time::Duration;

use crate::*;

pub struct DirectFlood {
    peer: Arc<Peer>,
    peer_store: Arc<PeerStore>,
    period: u64,
}

impl DirectFlood {
    pub fn new(peer: Arc<Peer>, peer_store: Arc<PeerStore>, period: u64) -> Arc<Self> {
        Arc::new(DirectFlood {
            peer,
            peer_store,
            period,
        })
    }

    pub async fn start(self: Arc<Self>) -> Result<()> {
        let message = Message::new(1, self.peer.get_addr(), Some("Yo!".into()));
        task::spawn(async move {
            let interval = Duration::from_secs(self.period);
            loop {
                task::sleep(interval).await;
                let this = self.clone();
                if let Err(e) = this.send_message(message.clone()).await {
                    eprintln!("Failed to broadcast message: {}", e);
                }
            }
        });
        Ok(())
    }

    pub async fn send_message(&self, message: Message) -> Result<()> {
        let peer_addrs = self
            .peer_store
            .get_peers()
            .await
            .keys()
            .map(|addr| addr.to_string())
            .collect::<Vec<_>>()
            .join("\", \"");
        log(format!(
            "Sending message \"{}\" to [\"{}\"]",
            message.clone().payload.unwrap(),
            peer_addrs
        ));
        let buffer: [u8; Peer::BUFFER_SIZE] = MessageKind::Direct(message).into();
        for (_, mut stream) in self.peer_store.clone().get_peers().await {
            if let Err(e) = stream.write_all(&buffer).await {
                eprintln!("Failed to broadcast message: {}", e);
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl Protocol for DirectFlood {
    async fn handle_message(&self, kind: &MessageKind) {
        if let MessageKind::Direct(message) = kind {
            log(format!(
                "Recieved message \"{}\" from \"{}\"",
                message.payload.clone().unwrap(),
                message.from
            ));
        }
    }
}
