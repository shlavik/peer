use async_std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Message {
    pub source: SocketAddr,
    pub payload: String,
}

impl Message {
    pub fn new(source: SocketAddr, payload: String) -> Self {
        Self { source, payload }
    }
}

impl From<Message> for [u8; 1024] {
    fn from(message: Message) -> Self {
        let mut buffer = [0u8; 1024];
        match message.source.ip() {
            IpAddr::V4(ipv4) => {
                buffer[0] = 4;
                buffer[1..5].copy_from_slice(&ipv4.octets());
            }
            IpAddr::V6(ipv6) => {
                buffer[0] = 6;
                buffer[1..17].copy_from_slice(&ipv6.octets());
            }
        }
        let port = message.source.port().to_be_bytes();
        let port_start = if buffer[0] == 4 { 5 } else { 17 };
        buffer[port_start..port_start + 2].copy_from_slice(&port);
        let payload_bytes = message.payload.as_bytes();
        let payload_length = payload_bytes.len().min(1004 - port_start - 2);
        buffer[port_start + 2..port_start + 2 + payload_length]
            .copy_from_slice(&payload_bytes[..payload_length]);
        buffer
    }
}

impl From<[u8; 1024]> for Message {
    fn from(buffer: [u8; 1024]) -> Self {
        let ip = match buffer[0] {
            4 => {
                let mut octets = [0u8; 4];
                octets.copy_from_slice(&buffer[1..5]);
                IpAddr::V4(Ipv4Addr::from(octets))
            }
            6 => {
                let mut octets = [0u8; 16];
                octets.copy_from_slice(&buffer[1..17]);
                IpAddr::V6(Ipv6Addr::from(octets))
            }
            _ => panic!("Bad IP address"),
        };
        let port_start = if buffer[0] == 4 { 5 } else { 17 };
        let port = u16::from_be_bytes(buffer[port_start..port_start + 2].try_into().unwrap());
        let payload_start = port_start + 2;
        let payload_length = buffer.len() - payload_start;
        let payload =
            String::from_utf8_lossy(&buffer[payload_start..payload_start + payload_length])
                .to_string();
        Message {
            source: SocketAddr::new(ip, port),
            payload,
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Message({} | {})", self.source, self.payload)
    }
}
