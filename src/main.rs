mod error;
mod agent;

pub use agent::*;

fn main() {
    let desc = AgentDescription::new("192.168.1.154:83".parse().unwrap(), "chicken pie")
                                .expect("error! name is probably too long");

    let serialized = desc.serialize();

    println!("Length: {}\nObject: {:?}\nData: {:?}", serialized.len(), AgentDescription::deserialize(&serialized), serialized);

}


