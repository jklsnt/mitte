use std::net::UdpSocket; // import std::net::UdpSocket
use std::net::SocketAddr;
use stunclient::StunClient;

fn main() { // the function that cargo run runs
    let local_addr: SocketAddr = "0.0.0.0:0".parse().unwrap(); // set let to be of type SocketAddr

    // upd communications protocol
    let udp = UdpSocket::bind(local_addr).unwrap(); // 

    let stunclient = StunClient::with_google_stun_server();
    let external_address = stunclient.query_external_address(&udp).unwrap();

    println!("{}", external_address);
}

