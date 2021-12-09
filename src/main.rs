mod error;
mod agent;

pub use agent::*;

fn main() {
    let desc = AgentDescription::new("0.0.0.0:0".parse().unwrap(), "aaaaaaaaaaaaaa")
                                .expect("error! name is probably too long");

    let serialized = desc.serialize();

    println!("Length: {}\nObject: {:?}", serialized.len(), AgentDescription::deserialize(&serialized));

}


