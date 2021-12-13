//! Establishes information regarding agents, which incl. both
//! sending and recieving partners

use super::error::*;

use std::time::Duration;

use bincode;
use serde::{Serialize, Deserialize};

use rand::rngs::OsRng;
use std::net::{UdpSocket, SocketAddrV4};
//use rsa::{RsaPublicKey, RsaPrivateKey};
use rsa::{PublicKey, RsaPrivateKey, RsaPublicKey, PaddingScheme};

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
    addr: Option<SocketAddrV4>,
    key: RsaPublicKey,
    pub name: String,
}

impl AgentDescription {

    /// Creates an agent. The agent's name must be smaller than or equal to 20 chars.
    ///
    /// # Arguments
    /// - `addr:&str`: the IPv4 Socket address which describes the location of agent
    /// - `name:&str`: the name of the agent. Must be <= 20 chars
    /// - `u8:&[u8]`: slice of U8 representing a bincode serialized `RsaPublcKey`
    ///
    /// # Returns
    /// `Result<Self, MitteError>`: potentially an instance of AgentDescription 
    pub fn new(addr:&str, name: &str, key: &[u8]) -> Result<Self, MitteError> {
        if name.len() > 20 {
            return Err(MitteError::DescriptionFormatError(String::from("name too long")));
        }

        let key:RsaPublicKey = match bincode::deserialize(key)
        { Ok(a) => a,
          Err(_) => {
              return Err(MitteError::DescriptionFormatError(
                  String::from("cannot parse public key")
              ))
          }};

        let address = addr.parse();
        match address {
            Ok(a) => Ok(AgentDescription {addr: Some(a), name: String::from(name), key}),
            Err(_) => Err(MitteError::DescriptionFormatError(String::from("cannot parse socket address")))
        }
    }

    /// Serialize the present `AgentDescription` object. 
    ///
    /// # Returns
    /// `Vec<u8>`: a vector of length 35 (if length incorrect, its padded)
    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = bincode::serialize(self).unwrap();

        while serialized.len() < 320 {
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

    /// Compares two agents and ensures that they are "the same" 
    ///
    /// similarity is determined by same name and same key
    ///
    /// # Returns
    /// `bool`: whether agents are the same
    pub fn is_same(&self, other:Self) -> bool {
        self.key == other.key && self.name == other.name
    }
}

impl PartialEq for AgentDescription {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key &&
            self.name == other.name &&
            self.addr == other.addr
    }
}

impl Eq for AgentDescription {}

// Don't quite know, but the initializer has to
// be a function
fn noneifier() -> Option<UdpSocket> { None }

#[derive(Serialize, Deserialize, Debug)]
pub struct Agent {
    pub profile: AgentDescription,
    peers: Vec<AgentDescription>,
    secret: RsaPrivateKey,

    #[serde(skip, default="noneifier")] 
    socket: Option<UdpSocket>,
}

impl Agent {
    pub fn new(addr:&str, name: &str) -> Result<Self, MitteError> {
        let priv_key = if let Ok(k) = RsaPrivateKey::new(&mut OsRng, 2048) { k }
        else {return Err(MitteError::AgentCreationError(String::from("cannot create key")))};

        let pub_key_serialized = bincode::serialize(&RsaPublicKey::from(&priv_key)).unwrap();
        let profile = AgentDescription::new(addr, name, &pub_key_serialized)?;

        let socket = UdpSocket::bind(profile.addr.expect("fatal: agent-created desc. does not have address"));
        match socket {
            Ok(s) => Ok(Agent { profile, peers: vec![], socket:Some(s), secret:priv_key}),
            Err(_) => Err(MitteError::AgentCreationError(String::from("cannot bind to socket")))
        }
    }

    /// Automatically bind to the descripted UDP socket if not bound, otherwise do nothing
    ///
    /// # Returns
    /// `Result<(), MitteError>`: nothing, or a failure
    fn autobind(&mut self) -> Result<(), MitteError> {
        if let None = self.socket {
            let socket_option = UdpSocket::bind(self.profile.addr.unwrap());

            match socket_option {
                Ok(s) => {
                    Ok(self.socket = Some(s))
                },
                Err(_) => {
                    Err(MitteError::HandshakeError(String::from("cannot bind to socket")))
                }
            }
        } else { Ok(()) }
    }

    pub fn handshake(&mut self, target: &AgentDescription) -> Result<(), MitteError> {
        // The handshake subrutine is a very long subroutine therefore, we shall attempt to
        // illustrate parts of it.

        // We begin by either getting or rebinding the socket if the socket was
        // no longer bound
        self.autobind()?;

        // Ensure that the socket is bound.
        if let Some(socket) = &self.socket {
            // Before we begin, we set the block time for read and write operations to one second
            // long. We don't want to wait too long for our peer to respond, and we will give up
            // if things take too long. We also save the original timeouts to write them back.
            let second = Duration::new(1,0);
            let old_read_timeout = socket.read_timeout().unwrap();
            let old_write_timeout = socket.write_timeout().unwrap();

            socket.set_read_timeout(Some(second)).unwrap();
            socket.set_write_timeout(Some(second)).unwrap();

            // We first attempt to connect to our target peer
            match socket.connect(target.addr.unwrap()) {
                Ok(_) => (),
                Err(_) => { return Err(MitteError::HandshakeError(String::from("peer disconnected"))); }
            }

            // We then send our mating message inviting to bind, telling nothing about ourselves
            // it looks very simple: 0 0 0 0 0 0 0, just 8 zeros
            socket.send(&[0;8]).unwrap(); 

            // We now hope that we get an acknowledge message back, that would be good so we could
            // introduce ourselves. The ack mesage is eight eights: 8 8 8 8 8 8 8 8
            let mut buf = [0;8]; // initialize a buffer of 8 zeros
            socket.recv(&mut buf).unwrap();

            // Check whether or not we actually got eight eights back
            if buf != [8;8] {
                return Err(MitteError::HandshakeError(String::from("handshake unacknowledged")));
            }

            // Ok, its time to tell our peer a little bit about ourselves
            // i.e. send them our agent description
            let desc = self.profile.serialize();
            socket.send(&desc).unwrap(); 

            // We now try to recieve four things, which has the shape of
            // 1 x y 1. This is the reciept acknowledgement. x, y are encoded
            // as follows
            //
            // 1. x - 1 (accept) 0 (reject)
            // 2. y - 1 (new connection) 0 (previous connection)
            let mut buf = [0;4]; // initialize a buffer of 4 zeros 
            socket.recv(&mut buf).unwrap();

            // We first check that the ack package is correctly 1-padded
            if !(buf[0] == buf[3] && buf[3] == 1) {
                return Err(MitteError::HandshakeError(String::from("handshake unacknowledged")));
            }

            // We then check that the ack has not been rejected 

            // We now set the original timeouts back
            socket.set_read_timeout(old_read_timeout).unwrap();
            socket.set_write_timeout(old_write_timeout).unwrap();

            return Ok(());
        } else {
            return Err(MitteError::HandshakeError(String::from("socket unbound")));
        }
    }

    pub fn listen(&mut self, target: &AgentDescription) -> Result<(), MitteError> {
        self.autobind()?;
        if let Some(socket) = &self.socket {

            let second = Duration::new(1,0);
            let old_read_timeout = socket.read_timeout().unwrap();
            let old_write_timeout = socket.write_timeout().unwrap();

            socket.set_read_timeout(Some(second)).unwrap();
            socket.set_write_timeout(Some(second)).unwrap();

            //match socket.connect(target.addr.unwrap()) {
            //    Ok(_) => (),
            //    Err(_) => { return Err(MitteError::HandshakeError(String::from("peer disconnected"))); }
            //}

            let mut buf = [1;8]; // initialize a buffer of 8 zeros
            socket.recv_from(&mut buf).unwrap();

            if buf != [0;8] {
                return Err(MitteError::HandshakeError(String::from("handshake unacknowledged")));
            }

            match socket.connect(target.addr.unwrap()) {
                Ok(_) => (),
                Err(_) => { return Err(MitteError::HandshakeError(String::from("cannot listen"))); }
            }

            socket.send(&[0;8]).unwrap();

            let mut peer_desc = [0;320];
            socket.recv(&mut peer_desc).unwrap();
            let peer = AgentDescription::deserialize(&peer_desc);

            let mut is_new = 1;
            if self.peers.contains(&peer) {
                is_new = 0;
            }

            let buf = [1, 1, is_new, 1]; // initialize a buffer of 4 zeros
            socket.send(&buf).unwrap();

            return Ok(());

        } else {
            return Err(MitteError::HandshakeError(String::from("socket unbound")));
        }

    }



    //let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    //let public_key = RsaPublicKey::from(&private_key);


    pub fn send_message(&mut self, msg: &[u8], name: String) -> Result<(), MitteError> {
        let mut rng = OsRng;
        self.autobind()?;

        let mut target_idx; // something wrong here! 
        match self.peers.iter().position(|r| r.name ==  name) {
            Some(v) => { target_idx = v; },
            None => { return Err(MitteError::HandshakeError(String::from("name is not in peers list"))); }
        }

        let peer_pub_key = &self.peers[target_idx].key;


        if let Some(socket) = &self.socket {
            match socket.connect(&self.peers[target_idx].addr.unwrap()) {
                Ok(_) => (),
                Err(_) => { return Err(MitteError::HandshakeError(String::from("peer disconnected"))); }
            }

            // encrypt the message

            //let padding = PaddingScheme::new_oaep::<sha2::Sha256>();
            //
            let padding = PaddingScheme::new_pkcs1v15_encrypt();
            let padding2 = PaddingScheme::new_pkcs1v15_encrypt();

            let mut enc_data = peer_pub_key.encrypt(&mut rng, padding, &msg[..]).expect("failed to encrypt");
            enc_data = RsaPrivateKey::sign(&self.secret, padding2, &enc_data).unwrap();

            //let enc_data = peer_pub_key.encrypt(&mut rng, padding, &msg[..]).expect("failed to encrypt");


            socket.send(&enc_data).unwrap();
            return Ok(());
        } else {
            return Err(MitteError::HandshakeError(String::from("socket unbound")));
        }
    }
}






















