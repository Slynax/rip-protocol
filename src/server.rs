use std::net::UdpSocket;
use std::time::Duration;
use crate::routing::{RipMessage, RoutingTable};

#[derive(Debug)]
pub struct RipServer {
    pub socket: UdpSocket,
    pub routing_table: RoutingTable,
}

impl RipServer {
    pub fn new(bind_addr: &str) -> Self {
        let socket = UdpSocket::bind(bind_addr).expect("Failed to bind socket");
        socket.set_read_timeout(Some(Duration::from_secs(1))).expect("Failed to set read timeout");
        RipServer {
            socket,
            routing_table: RoutingTable::new(),
        }
    }

    pub fn receive_rip_message(&mut self) {
        let mut buf = [0; 1024];
        if let Ok((amt, _src)) = self.socket.recv_from(&mut buf) {
            let received_data = &buf[..amt];
            if let Ok(message) = bincode::deserialize::<RipMessage>(received_data) {
                for route in message.routes {
                    self.routing_table.update(route.destination, message.sender.clone(), route.metric + 1, route.mask, route.out_interface_ip.clone());
                }
            }
        }
    }
}
