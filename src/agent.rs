//! Establishes information regarding agents, which incl. both
//! sending and recieving partners

use super::error::*;
use std::net::{UdpSocket, SocketAddrV4};
use serde::{Serialize, Deserialize};
use bincode;

/// A description for a given agent, including its name and address
///
/// # Examples
///
/// ```
/// use std::net::{SocketAddrV4, Ipv4Addr}
///
/// let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
/// let name = String::from("A Friend"); 
///
/// let desc = AgentDescription {addr: socket, name}
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct AgentDescription {
    pub addr: SocketAddrV4,
    name: String
}

impl AgentDescription {
    pub fn new(addr:SocketAddrV4, name: &str) -> Result<Self, MitteError> {
        if name.len() > 20 {
            return Err(MitteError::DescriptionFormatError(String::from("name too long")));
        }

        return Ok(AgentDescription {addr, name: String::from(name)});

    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = bincode::serialize(self).unwrap();

        while serialized.len() < 35 {
            serialized.push(0);
        }

        return serialized;
    }

    pub fn deserialize(v:&[u8]) -> Self {
        bincode::deserialize(v).unwrap()
    }
}

#[derive(Debug)]
pub struct Agent {
    pub profile: AgentDescription,
    peers: Vec<AgentDescription>
}

impl Agent {
    pub fn new(profile: AgentDescription) -> Self {
        Agent { profile, peers: vec![] } 
    }

    pub fn handshake(socket: &UdpSocket, target: &str) -> Result<(), MitteError> {
        match socket.connect(target) {
            Ok(_) => {},
            Err(_) => {return Err(MitteError::HandshakeError(String::from("disconnected")));}
        }
        
        let message_buffer:[u8;5] = [1,3,3,1,2];
        socket.send(&message_buffer).unwrap();

        return Ok(());
    }
}

