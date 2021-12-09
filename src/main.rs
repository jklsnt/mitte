mod error;
mod agent;

pub use agent::*;

fn main() {
    let desc = AgentDescription::new("0.0.0.0:0".parse().unwrap(), "aaaaaaaaaaaaaa")
                                .expect("error! name is probably too long");

    let serialized = desc.serialize();

    println!("Length: {}\nObject: {:?}", serialized.len(), AgentDescription::deserialize(&serialized));


    // Bind a UDP Port to listen on and find its external address
    // let udp = UdpSocket::bind("0.0.0.0:29131").unwrap();

    // handshake(&udp, "0.0.0.0:29130").unwrap();

    // // Connect to the UDP socket
    // udp.connect("0.0.0.0:29130").unwrap();

    // println!("Sending message!");
    // let message_buffer = [3,3];
    // udp.send(&message_buffer).unwrap();

    // println!("Recieving message!");
    // let mut message_reciever = [0;80];
    // udp.recv_from(&mut message_reciever).unwrap();

    // println!("Done! {:?}", &message_reciever);
}


