//use std::net::{UdpSocket, SocketAddr, SocketAddrV4, Ipv4Addr};
//use stunclient::StunClient;
use std::net::UdpSocket;
use stunclient::StunClient;
use std::net::{SocketAddr,ToSocketAddrs};

fn p<T:std::fmt::Debug>(test:T) { println!("{:?}", test) }

fn main() {
    let local_addr : SocketAddr = "0.0.0.0:0".parse().unwrap();
    let stun_addr = "stun.l.google.com:19302".to_socket_addrs().unwrap().filter(|x|x.is_ipv4()).next().unwrap();
    let udp = UdpSocket::bind(local_addr).unwrap();

    let c = StunClient::new(stun_addr);

    let my_external_addr = c.query_external_address(&udp).unwrap();
    p(my_external_addr);

    // RECEIVING
    println!("Recieving message!");
    let mut message_reciever = [0;10];
    udp.recv_from(&mut message_reciever).unwrap();
    println!("Done! {:?}", &message_reciever);



    /*
    // Define the STUN server
    // (this is the address of stun.gmx.net:3478)
    let socket = SocketAddrV4::new(Ipv4Addr::new(212,227,67,34), 3478);
    // Define the STUN Client
    let stunclient = StunClient::new(SocketAddr::V4(socket));

    // Bind a UDP Port to listen on and find its external address
    let udp = UdpSocket::bind("0.0.0.0:31021").unwrap();
    let external_address = stunclient.query_external_address(&udp).unwrap();

    println!("{}", external_address);

    udp.connect("10.1.130.13:29131").unwrap();

    // SENDING
    println!("Sending message!");
    let message_buffer = [8;10];
    udp.send(&message_buffer).unwrap();


    // // let (size, src) = udp.recv_from(&mut message_buffer).unwrap();
    // // let (_, _) = udp.recv_from(&mut message_buffer).unwrap();
    // udp.send_to(&message_buffer, "50.207.106.34:57031").unwrap();

    // println!("{:?}", message_reciever);
    */



}

