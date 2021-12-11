mod error;
mod agent;

pub use agent::*;

// use rsa::{RsaPrivateKey, RsaPublicKey};

mod debug;

fn main() {
    // let priv_key:RsaPrivateKey = bincode::deserialize(&debug::_KEY_PRIV).unwrap();
    // let pub_key:RsaPublicKey = bincode::deserialize(&debug::_KEY_PUB).unwrap();
    

    // let desc = AgentDescription::new("0.0.0.0:8301", "TestAgent")
    //                             .expect("error! name is probably too long");

    // let ag = Agent::new("0.0.0.0:8381", "TestAgent");
    // println!("{:?}", ag);

}

// let serialized = desc.serialize();

// println!("Length: {}\nObject: {:?}\nData: {:?}", serialized.len(), AgentDescription::deserialize(&serialized), serialized);




