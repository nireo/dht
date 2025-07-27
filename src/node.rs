use std::{
    fmt,
    net::{IpAddr, SocketAddr},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId([u8; 20]); // 160 bits = 20 bytes

impl NodeId {
    pub fn new(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() == 20 {
            let mut bytes = [0u8; 20];
            bytes.copy_from_slice(slice);
            Some(Self(bytes))
        } else {
            None
        }
    }

    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        let mut bytes = [0u8; 20];
        rng.fill(&mut bytes);
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    pub fn distance(&self, other: &NodeId) -> NodeId {
        let mut result = [0u8; 20];
        for i in 0..20 {
            result[i] = self.0[i] ^ other.0[i];
        }
        NodeId(result)
    }

    pub fn leading_zeros(&self) -> u32 {
        for (i, &byte) in self.0.iter().enumerate() {
            if byte != 0 {
                return (i as u32) * 8 + byte.leading_zeros();
            }
        }
        160
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub ip: Option<IpAddr>,
    pub port: Option<u16>,
}

impl Node {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            id: node_id,
            ip: None,
            port: None,
        }
    }

    pub fn with_address(node_id: NodeId, ip: IpAddr, port: u16) -> Self {
        Self {
            id: node_id,
            ip: Some(ip),
            port: Some(port),
        }
    }

    pub fn from_socket_addr(addr: SocketAddr) -> Self {
        Self {
            id: NodeId::random(),
            ip: Some(addr.ip()),
            port: Some(addr.port()),
        }
    }

    pub fn same_home_as(&self, other: &Node) -> bool {
        self.ip == other.ip && self.port == other.port
    }

    pub fn distance_to(&self, other: &Node) -> NodeId {
        self.id.distance(&other.id)
    }

    pub fn socket_addr(&self) -> Option<SocketAddr> {
        match (self.ip, self.port) {
            (Some(ip), Some(port)) => Some(SocketAddr::new(ip, port)),
            _ => None,
        }
    }

    pub fn has_address(&self) -> bool {
        self.ip.is_some() && self.port.is_some()
    }

    pub fn as_tuple(&self) -> (NodeId, Option<IpAddr>, Option<u16>) {
        (self.id, self.ip, self.port)
    }
}
