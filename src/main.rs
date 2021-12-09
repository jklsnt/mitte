mod error;
mod agent;

pub use agent::*;

fn main() {
    let desc = AgentDescription::new("0.0.0.0:8301", "TestAgent")
                                .expect("error! name is probably too long");

    let ag = Agent::new(desc);

    println!("{:?}", ag);
}

// let serialized = desc.serialize();

// println!("Length: {}\nObject: {:?}\nData: {:?}", serialized.len(), AgentDescription::deserialize(&serialized), serialized);




