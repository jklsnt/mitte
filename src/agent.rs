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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentDescription {
    addr: SocketAddrV4,
    name: String
}

impl AgentDescription {

    /// Creates an agent. The agent's name must be smaller than or equal to 20 chars.
    ///
    /// # Arguments
    /// - `addr:&str`: the IPv4 Socket address which describes the location of agent
    /// - `name:&str`: the name of the agent. Must be <= 20 chars
    ///
    /// # Returns
    /// `Result<Self, MitteError>`: potentially an instance of AgentDescription 
    pub fn new(addr:&str, name: &str) -> Result<Self, MitteError> {
        if name.len() > 20 {
            return Err(MitteError::DescriptionFormatError(String::from("name too long")));
        }

        let address = addr.parse();
        match address {
            Ok(a) => Ok(AgentDescription {addr: a, name: String::from(name)}),
            Err(_) => Err(MitteError::DescriptionFormatError(String::from("cannot parse socket address")))
        }
    }

    /// Serialize the present `AgentDescription` object. 
    ///
    /// # Returns
    /// `Vec<u8>`: a vector of length 35 (if length incorrect, its padded)
    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = bincode::serialize(self).unwrap();

        while serialized.len() < 35 {
            serialized.push(0);
        }

        return serialized;
    }

    /// Deserialize a bincode vector into an AgentDescription Object
    ///
    /// TODO actually verify what we get is an AgentDescription
    ///
    /// # Returns
    /// `AgentDescription`: the deserialized object
    pub fn deserialize(v:&[u8]) -> Self {
        bincode::deserialize(v).unwrap()
    }
}

#[derive(Debug)]
pub struct Agent {
    pub profile: AgentDescription,
    peers: Vec<AgentDescription>,
    socket: UdpSocket
}

impl Agent {
    pub fn new(profile: AgentDescription) -> Result<Self, MitteError> {
        let socket = UdpSocket::bind(profile.addr);
        match socket {
            Ok(s) => Ok(Agent { profile, peers: vec![], socket:s}),
            Err(_) => Err(MitteError::AgentCreationError(String::from("cannot bind to socket")))
        }
    }

    pub fn handshake(&mut self, socket: &UdpSocket, target: &AgentDescription) -> Result<(), MitteError> {
        // Oh boy oh boy this maybe a very long subroutine
        // therefore, we shall attempt to illustrate parst of it.

        // We first attempt to connect to our target peer
        match socket.connect(target.addr) {
            Ok(_) => (), // if we could, move on. If not, return a Err.
            Err(_) => {return Err(MitteError::HandshakeError(String::from("peer socket disconnected")));}
        }
        
        let message_buffer:[u8;5] = [1,3,3,1,2];
        socket.send(&message_buffer).unwrap();

        return Ok(());
    }
}

