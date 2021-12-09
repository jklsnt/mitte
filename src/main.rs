mod error;
mod agent;

pub use agent::*;

fn main() {
    let desc = AgentDescription::new("0.0.0.0:8393", "TestAgent")
                                .expect("error! name is probably too long");

    // let mut ag = Agent::new(desc);
    // ag.handshake(, target: &AgentDescription)
}

// let serialized = desc.serialize();

// println!("Length: {}\nObject: {:?}\nData: {:?}", serialized.len(), AgentDescription::deserialize(&serialized), serialized);




