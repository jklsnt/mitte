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
}

impl PartialEq for AgentDescription {
    // similarity is determined by same name and same key
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key &&
        self.name == other.name 
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
            Ok(s) => Ok(Agent { profile,
                                peers: vec![],
                                socket:Some(s),
                                secret:priv_key}),

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
            if buf[1] == 0 {
                return Err(MitteError::HandshakeError(String::from("handshake rejected")));
            }

            // We then check whether it is a new connection
            // if so, we ensure that we have not seen the peer before + add them
            // if not, we ensure that we have + update them
            if buf[2] == 1 && !self.peers.contains(target) {
                // new connection
                self.peers.push(target.clone());
            } else if buf[2] == 0 && self.peers.contains(target) {
                // these next two lines may seem real silly, but
                // the point is that PartialEq on `AgentDescription`
                // is defined such that there is actually
                let mut vec_filtered = self.peers.clone()
                    .into_iter()
                    .filter(|v| v != target)
                    .collect::<Vec<AgentDescription>>();
                vec_filtered.push(target.clone());
                self.peers = vec_filtered;
            } else {
                // return an error if they claim we've met before but we've not
                return Err(MitteError::HandshakeError(String::from("handshake connection malformed")));
            }

            // We now set the original timeouts back
            socket.set_read_timeout(old_read_timeout).unwrap();
            socket.set_write_timeout(old_write_timeout).unwrap();

            return Ok(());
        } else {
            return Err(MitteError::HandshakeError(String::from("socket unbound")));
        }
    }

    pub fn listen(&mut self, wait:u64) -> Result<(), MitteError> {
        // Beginning the autobind procidure as in the case with handshaking
        self.autobind()?;

        if let Some(socket) = &self.socket {
            // We start by setting the durations and clearing their timeouts
            let second = Duration::new(wait,0);
            let old_read_timeout = socket.read_timeout().unwrap();
            let old_write_timeout = socket.write_timeout().unwrap();

            // And set the timeouts as usually
            socket.set_read_timeout(Some(second)).unwrap();
            socket.set_write_timeout(Some(second)).unwrap();

            // We first by waiting to recieve a buffer of 8 zeros to align
            let mut buf = [1;8]; // initialize a buffer of 8 zeros
            let (_, sender) = socket.recv_from(&mut buf).unwrap();

            // If we didn't get 8 zeros, give up. 
            if buf != [0;8] {
                return Err(MitteError::ListenError(String::from("malformed input")));
            }

            // We send to our original sender the ack message and continue 
            // to wait for their full description of themselves
            socket.send_to(&[8;8], sender).unwrap();
            
            // And now, we wait for the reciept of the description of our peer
            let mut peer_desc = [0;320];
            socket.recv_from(&mut peer_desc).unwrap();
            let peer = AgentDescription::deserialize(&peer_desc);

            // Make sure that our peer actually sent an address
            if let None = peer.addr {
                return Err(MitteError::ListenError(String::from("peer did not send address")));
            }

            // Check whether or not we have the peer in the peers list
            // if we do, swap out the peer with the one that we got
            // so that we could update the address if needed (i.e. if we
            // have a peer 
            let mut is_new = 1;
            if self.peers.contains(&peer) {
                is_new = 0;
                let mut vec_filtered = self.peers.clone()
                    .into_iter()
                    .filter(|v| v != &peer)
                    .collect::<Vec<AgentDescription>>();
                vec_filtered.push(peer.clone());
                self.peers = vec_filtered;
            } else {
                self.peers.push(peer.clone())
            }

            // We finally acknowledge the final sent message and be done
            let buf = [1, 1, is_new, 1]; // initialize a buffer of 4 zeros
            socket.send_to(&buf, sender).unwrap();

            // We now set the original timeouts back
            socket.set_read_timeout(old_read_timeout).unwrap();
            socket.set_write_timeout(old_write_timeout).unwrap();

            return Ok(());

        } else {
            return Err(MitteError::ListenError(String::from("socket unbound")));
        }

    }

    /// Sends a message to a target peer. 
    ///
    /// # Arguments
    /// - `msg:&[u8]`: the message you want to send, in the form of an arr of u8s
    /// - `peer_name:&str`: the name of the peer you want to send a message to. Handshake must
    /// already be completed
    ///
    /// # Returns
    /// `Result<(), MitteError>`: null, or an error
    pub fn send_message(&mut self, msg: &[u8], peer_name: &str) -> Result<(), MitteError> {
        // We first establish a random number source
        let mut rng = OsRng;

        // We then check that our UDP port is bound
        self.autobind()?;

        // If the message length is larger than 512 units, we consider it too long
        if msg.len() > 512 {
            return Err(MitteError::SendError(String::from("message too long")));
        }

        // We then match the correct peer to communicate with
        if let Some(peer) = self.peers.iter().filter(|r| r.name == peer_name).next() {

            // We also make sure that the socket is bound
            if let Some(socket) = &self.socket {

                // If connection with the peer was not successful, we error
                if let Err(_) = socket.connect(peer.addr.unwrap()) {
                    return Err(MitteError::SendError(String::from("peer disconnected"))); 
                }

                // We then encode the data as needed
                let padding = PaddingScheme::new_pkcs1v15_encrypt();
                let enc_data:Vec<u8> = peer.key.encrypt(&mut rng, padding, msg).unwrap();

                // Finally, we add establishment values 0 0 + length of the communication
                // this implementation of UDP only sends `u8`s, so we split the length up
                // into two u8s
                let data_len = enc_data.len() as u16;
                let (a,b) = ((data_len >> 8) as u8, data_len as u8);

                // We chunck the start digits + the bitshifted leng along
                let chained_data = [0,0,a,b] 
                    .iter()
                    .chain(enc_data.iter())
                    .cloned()
                    .collect::<Vec<u8>>();

                // Send it along!
                socket.send(&chained_data).unwrap();
                return Ok(());
            } else {
                return Err(MitteError::SendError(String::from("socket unbound")));
            }
        } else {
            return Err(MitteError::SendError(String::from("name is not in peers list"))); 
        }
    }

    /// Receives a single message. After receiving a message, the message is returned and the
    /// function quits.
    ///
    /// # Returns
    /// `Result<<Vec<u8>, MitteError>`: potentially the received, decrypted message
    pub fn recv_message(&mut self) -> Result<Vec<u8>, MitteError> {
        self.autobind()?;

        if let Some(socket) = &self.socket {
            // We first recieve a message
            let mut buf = [0;1024]; // TODO: len checks!
            socket.recv(&mut buf).unwrap();

            // We then check that the setup values are correct
            if buf[0] != buf[1] || buf[1] != 0 {
                return Err(MitteError::ReceiveError(String::from("incorrect setup values")));
            }

            // We then get the appropriate length for our data by bitshifting
            // it back (i.e. constructing a `u16` out of two `u8` because
            // UDP can't send `u16`s

            let len = ((buf[2] as u16) << 8 + buf[3]) as usize;

            // We use typical decoding schemes to decode it
            let padding = PaddingScheme::new_pkcs1v15_encrypt();
            match self.secret.decrypt(padding, &buf[4..len+4]) {
                Ok(d) => { Ok(d) },
                Err(_) => {
                    return Err(MitteError::ReceiveError(String::from("decryption error")));
                }
            }

        } else {
            return Err(MitteError::ReceiveError(String::from("socket unbound")));
        }
    }
}
























