use bincode::Options;
use log;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::net::SocketAddr;
use std::str;
use tokio::net::UdpSocket;

pub struct Client {
    socket: UdpSocket,
    max_datagram_size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct DNSMessage {
    id: [u8; 2],
    flags: [u8; 2],
    questions: [u8; 2],
    answers_rrs: [u8; 2],
    authority_rrs: [u8; 2],
    additional_rrs: [u8; 2],
    queries: Vec<u8>,
    answers: Vec<u8>,
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("ParseError")]
    ParseError(String),
    #[error("BindError")]
    BindError(String),
    #[error("ConnectError")]
    ConnectError(String),
    #[error("SendError")]
    SendError(String),
    #[error("RecvError")]
    RecvError(String),
    #[error("EncodeError")]
    EncodeError(String),
    #[error("DecodeError")]
    DecodeError(String),
    #[error("DecodeIdError")]
    DecodeIdError(String),

    #[error("DNS message RDCode format error")]
    RDCodeFormatError,
    #[error("DNS message RDCode server failure error")]
    RDCodeServerFailure,
    #[error("DNS message RDCode name error")]
    RDCodeNameError,
    #[error("DNS message RDCode not implemented server error")]
    RDCodeNotImplemented,
    #[error("DNS message RDCode server refused error")]
    RDCodeRefused,
}

#[derive(Debug, PartialEq)]
pub enum QueryType {
    A,
    AAAA,
    SOA,
    CNAME,
}

#[derive(Debug)]
enum ClassType {
    IN,
}

#[derive(Debug)]
pub struct QueryAnswer {
    host: String,
    address: String,
    query_type: QueryType,
    class_type: ClassType,
}

impl DNSMessage {
    /// DNS UDP header size: id + flags + questions + answers_rrs + authority_rrs +
    /// additional_rss
    fn header_size() -> usize {
        12
    }

    fn new(queries: Vec<u8>) -> DNSMessage {
        let id: u16 = random();
        DNSMessage {
            id: [(id >> 8) as u8, (id & 0xff) as u8],
            flags: [1, 0],
            questions: [0, 1],
            answers_rrs: [0, 0],
            authority_rrs: [0, 0],
            additional_rrs: [0, 0],
            queries: queries,
            answers: Vec::new(),
        }
    }

    fn encode(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let bincode_opts = bincode::DefaultOptions::new()
            .with_big_endian()
            .with_no_limit()
            .with_varint_encoding();
        let mut encoded = bincode_opts.serialize(&self)?;
        // remove Vec's len from encoded bytes
        encoded.remove(DNSMessage::header_size());
        Ok(encoded)
    }

    fn encode_query_type(query_type: &QueryType) -> Vec<u8> {
        match query_type {
            QueryType::A => vec![0, 1],
            QueryType::AAAA => vec![0, 0x1c],
            _ => vec![],
        }
    }

    fn decode_query_type(values: &[u8]) -> Result<QueryType, ClientError> {
        match values {
            [0, 1] => Ok(QueryType::A),
            [0, 0x1c] => Ok(QueryType::AAAA),
            [0, 5] => Ok(QueryType::CNAME),
            [0, 6] => Ok(QueryType::SOA),
            _ => Err(ClientError::DecodeError(std::format!(
                "Failed to decode query type {:x?}",
                &values
            ))),
        }
    }

    fn encode_class_type() -> Vec<u8> {
        vec![0, 1] // IN
    }

    fn decode_class_type(values: &[u8]) -> Result<ClassType, ClientError> {
        match values {
            [0, 1] => Ok(ClassType::IN),
            _ => Err(ClientError::DecodeError(std::format!(
                "Failed to decode class type {:x?}",
                &values
            ))),
        }
    }

    fn encode_host(host: &String, query_type: &QueryType) -> Vec<u8> {
        let mut encoded: Vec<u8> = Vec::new();
        for word in host.split(".") {
            encoded.push(word.len() as u8);
            for bytes in word.as_bytes() {
                encoded.push(*bytes);
            }
        }
        // end of word
        encoded.push(0x0);

        encoded.extend(DNSMessage::encode_query_type(&query_type));
        encoded.extend(DNSMessage::encode_class_type());
        encoded
    }

    fn decode(&self, data: &Vec<u8>, rcvd_len: usize) -> Result<Self, Box<dyn Error>> {
        let bincode_opts = bincode::DefaultOptions::new()
            .with_big_endian()
            .with_no_limit()
            .allow_trailing_bytes()
            .with_varint_encoding();
        let msg: DNSMessage = bincode_opts.deserialize(&data[..rcvd_len])?;
        Ok(msg)
    }

    fn decode_query_answers(
        &self,
        host: String,
        queries_len: usize,
        rest: &[u8],
    ) -> Result<Vec<QueryAnswer>, ClientError> {
        let rest_len = rest.len();
        let mut answers: Vec<QueryAnswer> = Vec::new();
        if rest_len < queries_len {
            return Ok(answers);
        }
        let resp = &rest[queries_len..rest_len];
        log::debug!("Decoding query answers: {:x?}", &resp);
        let mut i: usize = 0;
        while i + 12 < resp.len() {
            if resp[i] != 0xc0 {
                return Err(ClientError::DecodeError(std::format!(
                    "Expected 0xc0 on decoded response, found {:x?} instead",
                    &resp[i]
                )));
            }
            let query_type = DNSMessage::decode_query_type(&resp[i + 2..i + 4])?;
            let class_type = DNSMessage::decode_class_type(&resp[i + 4..i + 6])?;
            let _ttl = &resp[i + 6..i + 10];
            let data_len: u16 = ((resp[i + 10] as u16) << 8) | resp[i + 11] as u16;
            if !(query_type == QueryType::A || query_type == QueryType::AAAA) {
                i = i + 12 + (data_len as usize);
                continue;
            }
            let answer = QueryAnswer {
                host: host.clone(),
                address: if query_type == QueryType::A {
                    Ipv4Addr::new(resp[i + 12], resp[i + 13], resp[i + 14], resp[i + 15])
                        .to_string()
                } else {
                    Ipv6Addr::new(
                        ((resp[i + 12] as u16) << 8) | resp[i + 13] as u16,
                        ((resp[i + 14] as u16) << 8) | resp[i + 15] as u16,
                        ((resp[i + 16] as u16) << 8) | resp[i + 17] as u16,
                        ((resp[i + 18] as u16) << 8) | resp[i + 19] as u16,
                        ((resp[i + 20] as u16) << 8) | resp[i + 21] as u16,
                        ((resp[i + 22] as u16) << 8) | resp[i + 23] as u16,
                        ((resp[i + 24] as u16) << 8) | resp[i + 25] as u16,
                        ((resp[i + 26] as u16) << 8) | resp[i + 27] as u16,
                    )
                    .to_string()
                },
                query_type: query_type,
                class_type: class_type,
            };
            answers.push(answer);
            i = i + 12 + (data_len as usize);
        }
        Ok(answers)
    }

    fn is_answer(&self) -> bool {
        self.flags[0] & 0x80 == 0x80
    }

    fn is_query(&self) -> bool {
        self.flags[0] & 0x80 == 0x00
    }

    fn op_code(&self) -> u8 {
        // (self.flags[0] & 0x80) ;
        0
    }

    fn rd_code(&self) -> Result<(), ClientError> {
        match self.flags[1] & 0x0f {
            0 => Ok(()),
            1 => Err(ClientError::RDCodeFormatError),
            2 => Err(ClientError::RDCodeServerFailure),
            3 => Err(ClientError::RDCodeNameError),
            4 => Err(ClientError::RDCodeNotImplemented),
            5 => Err(ClientError::RDCodeRefused),
            _ => Ok(()),
        }
    }
}

impl Client {
    pub async fn new(remote_addr: String) -> Result<Client, ClientError> {
        let remote_addr: SocketAddr = match remote_addr.parse() {
            Ok(addr) => addr,
            Err(err) => return Err(ClientError::ParseError(err.to_string())),
        };
        let local_addr: SocketAddr = match if remote_addr.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        }
        .parse()
        {
            Ok(addr) => addr,
            Err(err) => return Err(ClientError::ParseError(err.to_string())),
        };
        let socket = match UdpSocket::bind(local_addr).await {
            Ok(socket) => socket,
            Err(err) => return Err(ClientError::BindError(err.to_string())),
        };
        let max_datagram_size: usize = 65_507;
        match socket.connect(&remote_addr).await {
            Ok(res) => res,
            Err(err) => return Err(ClientError::ConnectError(err.to_string())),
        };
        Ok(Client {
            socket,
            max_datagram_size,
        })
    }

    pub async fn query(
        &self,
        host: String,
        query_type: QueryType,
    ) -> Result<Vec<QueryAnswer>, ClientError> {
        let queries = DNSMessage::encode_host(&host, &query_type);
        let queries_len = queries.len();
        let msg = &DNSMessage::new(queries);
        log::debug!("Query {:x?}", msg);
        let msg_enc = match msg.encode() {
            Ok(encoded) => encoded,
            Err(err) => return Err(ClientError::EncodeError(err.to_string())),
        };
        match self.socket.send(&msg_enc).await {
            Ok(_) => (),
            Err(err) => return Err(ClientError::SendError(err.to_string())),
        };
        let mut data = vec![0u8; self.max_datagram_size];
        let len = match self.socket.recv(&mut data).await {
            Ok(len) => len,
            Err(err) => return Err(ClientError::RecvError(err.to_string())),
        };
        log::debug!("Query encoded {:x?}, received {:?} bytes", msg_enc, len);
        let msg_decoded = match msg.decode(&data, len) {
            Ok(decoded) => decoded,
            Err(err) => return Err(ClientError::DecodeError(err.to_string())),
        };
        let encode_size = DNSMessage::header_size();
        let rest = &data[encode_size..len];
        log::debug!("Rest {:x?}", rest);
        if msg.id != msg_decoded.id {
            let err_msg: String = std::format!(
                "Sent Query ID: {:?}, but received Response ID: {:?}",
                msg.id,
                msg_decoded.id
            );
            return Err(ClientError::DecodeIdError(err_msg));
        }
        log::debug!("Response {:x?}", &msg_decoded);
        match msg_decoded.rd_code() {
            Ok(()) => msg_decoded.decode_query_answers(host, queries_len, rest),
            Err(err) => Err(err),
        }
    }
}
