use std::net::UdpSocket;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use crate::routing::{RipMessage, RoutingTable};

#[derive(Debug)]
pub struct RipClient {
    pub socket: UdpSocket,
    pub routing_table: RoutingTable,
}

impl RipClient {
    pub fn new(bind_addr: &str) -> Self {
        let socket = UdpSocket::bind(bind_addr).expect("Failed to bind socket");
        socket.set_read_timeout(Some(Duration::from_secs(1))).expect("Failed to set read timeout");
        RipClient {
            socket,
            routing_table: RoutingTable::new(),
        }
    }

    pub fn send_rip_message(&self, dest_addr: &str, message: &RipMessage) {
        let serialized_message = bincode::serialize(&message).expect("Failed to serialize message");
        self.socket.send_to(&serialized_message, dest_addr).expect("Failed to send message");
    }
}
