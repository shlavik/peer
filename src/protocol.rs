use async_std::net::SocketAddr;

use crate::Peer;

#[async_trait::async_trait]
pub trait Protocol: Send + Sync {
    async fn handle_message(&self, message: &MessageKind);
}

#[derive(Debug, Clone)]
pub struct Message {
    pub ttl: u8,
    pub from: SocketAddr,
    pub payload: Option<String>,
}

impl Message {
    pub fn new(ttl: u8, from: SocketAddr, payload: Option<String>) -> Self {
        Message { from, ttl, payload }
    }
}

impl From<Message> for Vec<u8> {
    fn from(message: Message) -> Self {
        let mut bytes = Vec::new();
        bytes.push(message.ttl);
        let addr_str = message.from.to_string();
        let addr_len = addr_str.len() as u8;
        bytes.push(addr_len);
        bytes.extend(addr_str.as_bytes());
        match message.payload {
            None => bytes.push(0),
            Some(text) => {
                bytes.push(1);
                bytes.extend(Vec::from(text.as_bytes()));
            }
        }
        bytes
    }
}

impl From<Vec<u8>> for Message {
    fn from(bytes: Vec<u8>) -> Self {
        let ttl = bytes[0];
        let addr_len = bytes[1] as usize;
        let addr_str = String::from_utf8_lossy(&bytes[2..2 + addr_len]).to_string();
        let from = addr_str
            .parse::<SocketAddr>()
            .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 0)));
        let payload = if bytes[2 + addr_len] == 0 {
            None
        } else {
            let text = String::from_utf8_lossy(&bytes[3 + addr_len..])
                .trim_end_matches('\0')
                .to_string();
            Some(text)
        };
        Message { ttl, from, payload }
    }
}

#[derive(Debug, Clone)]
pub enum MessageKind {
    Handshake(Message),
    Direct(Message),
    Broadcast(Message),
    Ping,
}

impl From<MessageKind> for Vec<u8> {
    fn from(kind: MessageKind) -> Self {
        let mut bytes = Vec::new();
        match kind {
            MessageKind::Handshake(message) => {
                bytes.push(0);
                bytes.extend(Vec::from(message));
            }
            MessageKind::Direct(message) => {
                bytes.push(1);
                bytes.extend(Vec::from(message));
            }
            MessageKind::Broadcast(message) => {
                bytes.push(2);
                bytes.extend(Vec::from(message));
            }
            MessageKind::Ping => {
                bytes.push(255);
            }
        }
        bytes
    }
}

impl From<Vec<u8>> for MessageKind {
    fn from(bytes: Vec<u8>) -> Self {
        match bytes[0] {
            0 => MessageKind::Handshake(Message::from(Vec::from(&bytes[1..]))),
            1 => MessageKind::Direct(Message::from(Vec::from(&bytes[1..]))),
            2 => MessageKind::Broadcast(Message::from(Vec::from(&bytes[1..]))),
            _ => MessageKind::Ping,
        }
    }
}

impl From<MessageKind> for [u8; Peer::BUFFER_SIZE] {
    fn from(data: MessageKind) -> Self {
        let data_vec = Vec::from(data);
        let len = data_vec.len();
        if len > Peer::BUFFER_SIZE {
            panic!("Data does not fit into the buffer!");
        }
        let mut buffer = [0; Peer::BUFFER_SIZE];
        buffer[..len].copy_from_slice(&data_vec[..len]);
        buffer
    }
}

impl From<[u8; Peer::BUFFER_SIZE]> for MessageKind {
    fn from(bytes: [u8; Peer::BUFFER_SIZE]) -> Self {
        MessageKind::from(Vec::from(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    fn make_message_v4() -> Message {
        let ttl = 0;
        let from = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let payload = Some(String::from("1234567890"));
        Message { ttl, from, payload }
    }

    fn make_message_v6() -> Message {
        let ttl = 255;
        let from = SocketAddr::new(
            IpAddr::V6(Ipv6Addr::new(255, 255, 255, 255, 255, 255, 255, 255)),
            8080,
        );
        let payload = None;
        Message { ttl, from, payload }
    }

    #[test]
    fn test_message_to_bytes_and_back_v4() {
        let message_1 = make_message_v4();
        let bytes = Vec::<u8>::from(message_1.clone());
        let message_2 = Message::from(bytes);
        assert_eq!(message_1.from, message_2.from);
        assert_eq!(message_1.payload, message_2.payload);
    }

    #[test]
    fn test_message_to_bytes_and_back_v6() {
        let message_1 = make_message_v6();
        let bytes = Vec::<u8>::from(message_1.clone());
        let message_2 = Message::from(bytes);
        assert_eq!(message_1.from, message_2.from);
        assert_eq!(message_1.payload, message_2.payload);
    }

    #[test]
    fn test_message_kind_handshake_to_bytes_and_back_v4() {
        let message_1 = make_message_v4();
        let data_1 = MessageKind::Handshake(message_1.clone());
        let bytes = Vec::from(data_1.clone());
        let data_2 = MessageKind::from(bytes);
        match data_2 {
            MessageKind::Handshake(message_2) => {
                assert_eq!(message_2.from, message_1.from);
                assert_eq!(message_2.payload, message_1.payload);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_message_kind_broadcast_to_bytes_and_back_v6() {
        let message_1 = make_message_v6();
        let data_1 = MessageKind::Broadcast(message_1.clone());
        let bytes = Vec::from(data_1.clone());
        let data_2 = MessageKind::from(bytes);
        match data_2 {
            MessageKind::Broadcast(message_2) => {
                assert_eq!(message_2.from, message_1.from);
                assert_eq!(message_2.payload, message_1.payload);
            }
            _ => unreachable!(),
        }
    }
}
