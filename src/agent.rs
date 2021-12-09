//! Establishes information regarding agents, which incl. both
//! sending and recieving partners

use super::error::*;
use std::net::{UdpSocket, SocketAddrV4};
use serde::{Serialize, Deserialize};
use bincode;
use std::time::Duration;
use std::error::Error;

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

    pub fn handshake(&self, target: &AgentDescription) -> Result<(), Box<dyn Error>> {
        // The handshake subrutine is a very long subroutine therefore, we shall attempt to
        // illustrate parts of it.

        // Before we begin, we set the block time for read and write operations to one second
        // long. We don't want to wait too long for our peer to respond, and we will give up
        // if things take too long. We also save the original timeouts to write them back.
        let second = Duration::new(1,0);

        let old_read_timeout = self.socket.read_timeout().unwrap();
        let old_write_timeout = self.socket.write_timeout().unwrap();

        self.socket.set_read_timeout(Some(second)).unwrap();
        self.socket.set_write_timeout(Some(second)).unwrap();

        // We first attempt to connect to our target peer
        self.socket.connect(target.addr)?; 
        
        // We then send our mating message inviting to bind, telling nothing about ourselves
        // it looks very simple: 0 0 0 0 0 0 0, just 8 zeros
        self.socket.send(&[0;8])?; 

        // We now hope that we get an acknowledge message back, that would be good so we could
        // introduce ourselves. The ack nowledge is eitght eights: 8 8 8 8 8 8 8 8
        let mut buf = [0;8]; // initialize a buffer of 8 zeros
        self.socket.recv(&mut buf)?; 
        
        // We now set the original timeouts back
        self.socket.set_read_timeout(old_read_timeout).unwrap();
        self.socket.set_write_timeout(old_write_timeout).unwrap();

        return Ok(());
    }
}

