use std::net::{UdpSocket, Ipv4Addr};
use std::thread;
use std::time::Duration;
use crate::router::InterfaceWrapper;
use serde::{Serialize, Deserialize};
use core::net::SocketAddr;
use core::net::IpAddr;
use std::sync::{Arc, Mutex};

const RIP_PORT: u16 = 8080;

pub struct Router {
    config: Vec<InterfaceWrapper>,
    routing_table: Arc<Mutex<Vec<Route>>>,
    send_socket: Option<UdpSocket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub network: Ipv4Addr,
    pub mask: u8,
    pub next_hop: Ipv4Addr,
    pub metric: u8,
}

impl Router {
    pub fn new(config: Vec<InterfaceWrapper>) -> Self {
        let routing_table = Arc::new(Mutex::new(Vec::new()));
        let send_socket = None;

        for interface in &config {
            let ip = interface.interface.ip.parse::<Ipv4Addr>().expect("Failed to parse IP address");
            let mask = interface.interface.mask;
            let network = get_network(ip, mask);
            let route = Route {
                network,
                mask,
                next_hop: ip,
                metric: 1,
            };
            routing_table.lock().unwrap().push(route);
        }

        Router { config, routing_table, send_socket }
    }

    pub fn start(&mut self, ip: Ipv4Addr) {
        let config = self.config.clone();
        let send_socket = UdpSocket::bind((ip, 0)).expect("Failed to bind send socket");
        let recv_socket = UdpSocket::bind((ip, RIP_PORT)).expect("Failed to bind recv socket");

        // Store the send_socket in the Router for later use
        self.send_socket = Some(send_socket);

        // Clone the Arc for the receiver thread
        let routing_table = Arc::clone(&self.routing_table);

        // Start the receiver thread
        thread::spawn(move || {
            let mut buf = [0; 1024];
            loop {
                let (amt, _src) = recv_socket.recv_from(&mut buf).expect("Failed to receive data");
                let received_data = &buf[..amt];
                let mut routing_table = routing_table.lock().unwrap();
                update_routing_table(&mut routing_table, received_data);
            }
        });
    }

    pub fn send_update(&self, dest_ip: Ipv4Addr) {
        let send_socket = self.send_socket.as_ref().expect("Send socket not initialized");
        let dest = SocketAddr::new(IpAddr::V4(dest_ip), RIP_PORT);
    
        let routing_table = self.routing_table.lock().unwrap().clone();
        let data = bincode::serialize(&routing_table).expect("Failed to serialize data");
    
        send_socket.send_to(&data, &dest).expect("Failed to send data");
    }

    pub fn get_routing_table(&self) -> Vec<Route> {
        let routing_table = self.routing_table.lock().unwrap();
        routing_table.clone()
    }
}

fn update_routing_table(routing_table: &mut Vec<Route>, data: &[u8]) {
    let received_routes: Vec<Route> = bincode::deserialize(data).expect("Failed to deserialize data");

    for route in received_routes {
        add_or_update_route(routing_table, route);
    }
}

fn add_or_update_route(routing_table: &mut Vec<Route>, route: Route) {
    let existing_route = routing_table.iter().find(|r| r.network == route.network && r.mask == route.mask);
    if let Some(existing_route) = existing_route {
        if existing_route.metric > route.metric {
            routing_table.retain(|r| r.network != route.network || r.mask != route.mask);
            routing_table.push(route);
        }
    } else {
        routing_table.push(route);
    }
}

fn get_network(ip: Ipv4Addr, mask: u8) -> Ipv4Addr {
    let ip_int = u32::from(ip);
    let mask_int = !(0xFFFFFFFFu32 >> mask);
    let network_int = ip_int & mask_int;
    Ipv4Addr::from(network_int)
}