use std::net::{UdpSocket, SocketAddr, SocketAddrV4, Ipv4Addr};
use stunclient::StunClient;

fn main() {
    // Define the STUN server
    // (this is the address of stun.gmx.net:3478)
    let socket = SocketAddrV4::new(Ipv4Addr::new(212,227,67,34), 3478);
    // Define the STUN Client
    let stunclient = StunClient::new(SocketAddr::V4(socket));

    // Bind a UDP Port to listen on and find its external address
    let udp = UdpSocket::bind("0.0.0.0:31021").unwrap();
    let external_address = stunclient.query_external_address(&udp).unwrap();

    println!("{}", external_address);

    // udp.connect("98.42.49.47:31022").unwrap();

    // println!("Sending message!");
    // let message_buffer = [8;10];
    // udp.send(&message_buffer).unwrap();

    println!("Recieving message!");
    let mut message_reciever = [0;10];
    udp.recv_from(&mut message_reciever).unwrap();

    println!("Done! {:?}", &message_reciever);

    // // let (size, src) = udp.recv_from(&mut message_buffer).unwrap();
    // // let (_, _) = udp.recv_from(&mut message_buffer).unwrap();
    // udp.send_to(&message_buffer, "50.207.106.34:57031").unwrap();

    // println!("{:?}", message_reciever);
}

