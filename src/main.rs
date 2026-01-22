use deku::prelude::*;
use std::{
    fmt,
    net::{Ipv4Addr, UdpSocket},
};

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
struct Label {
    #[deku(endian = "big")]
    length: u8,
    #[deku(count = "length")]
    data: Vec<u8>,
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.data))
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
struct Name {
    #[deku(until = "|label: &Label| label.length == 0")]
    labels: Vec<Label>,
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.labels
                .iter()
                .map(|label| label.to_string())
                .collect::<Vec<String>>()
                .join(".")
        )
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone)]
struct Question {
    name: Name,
    #[deku(endian = "big")]
    _type: u16, // A Record Type
    #[deku(endian = "big")]
    class: u16, // IN record class
}

impl fmt::Display for Question {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{:?}", self.name, self._type, self.class)
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
struct Header {
    #[deku(endian = "big")]
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
    #[deku(endian = "big")]
    qdcount: u16, // Question Count
    #[deku(endian = "big")]
    ancount: u16, // Answer Record Count
    #[deku(endian = "big")]
    nscount: u16, // Authority Record Count
    #[deku(endian = "big")]
    arcount: u16, // Additional Record Count
    question: Question, // Question Section
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Header {{ id: {}, qr: {}, opcode: {}, aa: {}, tc: {}, rd: {}, ra: {}, z: {}, rcode: {}, qdcount: {}, ancount: {}, nscount: {}, arcount: {}, question: {} }}",
            self.id,
            self.qr,
            self.opcode,
            self.aa,
            self.tc,
            self.rd,
            self.ra,
            self.z,
            self.rcode,
            self.qdcount,
            self.ancount,
            self.nscount,
            self.arcount,
            self.question
        )
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
struct Answer {
    name: Name, // Label sequence
    #[deku(endian = "big")]
    _type: u16, // A record type
    #[deku(endian = "big")]
    class: u16, // IN record type
    #[deku(endian = "big")]
    ttl: u32, // Time to live
    #[deku(endian = "big")]
    length: u16, // Length of the data field
    #[deku(endian = "big")]
    data: Ipv4Addr,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
struct Response {
    header: Header,
    answers: Answer,
}

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                let (_rest, mut val) = Header::from_bytes((&buf, 0)).unwrap();
                val.id = 1234;
                val.qr = true;
                val.qdcount = 1;
                val.ancount = 1;
                val.question._type = 1;
                val.question.class = 1;
                println!("{}", &val);
                let answer = Answer {
                    name: val.question.name.clone(),
                    _type: 1,
                    class: 1,
                    ttl: 60,
                    length: 4,
                    data: Ipv4Addr::new(127, 0, 0, 1),
                };
                let response = Response {
                    header: val,
                    answers: answer,
                };
                udp_socket
                    .send_to(&response.to_bytes().unwrap(), source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
