use deku::prelude::*;
use std::net::UdpSocket;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
struct Header {
    id: u16, // Packet ID
    #[deku(bits = "1")]
    qr: bool, // Query/Response Indicator
    #[deku(bits = "4")]
    opcode: u8, // Operation Code
    #[deku(bits = "1")]
    aa: bool, // Authoritative Answer
    #[deku(bits = "1")]
    tc: bool, // Truncation
    #[deku(bits = "1")]
    rd: bool, // Recursion Desired
    #[deku(bits = "1")]
    ra: bool, // Recursion Available
    #[deku(bits = "3")]
    z: u8, // Reserved
    #[deku(bits = "4")]
    rcode: u8, // Response Code
    qdcount: u16, // Question Count
    ancount: u16, // Answer Record Count
    nscount: u16, // Authority Record Count
    arcount: u16, // Additional Record Count
}

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                let (_rest, mut val) = Header::from_bytes((&buf, 0)).unwrap();
                val.qr = true;
                println!("{:?}", &val);
                udp_socket
                    .send_to(&val.to_bytes().unwrap(), source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
