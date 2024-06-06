use std::net::{UdpSocket, Ipv4Addr};
use std::thread;
use crate::router::InterfaceWrapper;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use std::net::IpAddr;

const RIP_PORT: u16 = 8080;

pub struct Router {
    config: Vec<InterfaceWrapper>,
    routing_table: Arc<Mutex<Vec<Route>>>,
    send_socket: Option<UdpSocket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub network: Ipv4Addr,
    pub mask: Ipv4Addr,
    pub next_hop: Option<Ipv4Addr>,
    pub metric: u8,
    pub exit_interface: Ipv4Addr,
}

fn get_exit_interface(config: &[InterfaceWrapper], dest_ip: Ipv4Addr) -> Option<Ipv4Addr> {
    for interface in config {
        let ip = interface.interface.ip.parse::<Ipv4Addr>().expect("Failed to parse IP address");
        let mask = mask_to_ip(interface.interface.mask);
        let network = get_network(ip, mask);
        if network == get_network(dest_ip, mask) {
            return Some(ip);
        }
    }
    None
}

fn initialize_routes(config: &[InterfaceWrapper]) -> Vec<Route> {
    let mut routes = Vec::new();
    for interface in config {
        let ip = interface.interface.ip.parse::<Ipv4Addr>().expect("Failed to parse IP address");
        let mask = mask_to_ip(interface.interface.mask);
        let network = get_network(ip, mask);
        let route = Route {
            network,
            mask,
            next_hop: None,
            metric: 1,
            exit_interface: ip,
        };
        routes.push(route);
    }
    routes
}

impl Router {
    pub fn new(config: Vec<InterfaceWrapper>) -> Self {
        let routing_table = Arc::new(Mutex::new(initialize_routes(&config)));
        let send_socket = None;

        Router { config, routing_table, send_socket }
    }

    pub fn start(&mut self, ip: Ipv4Addr) {
        let config = self.config.clone();
        let send_socket = UdpSocket::bind((ip, 0)).expect("Failed to bind send socket");
        let recv_socket = UdpSocket::bind((ip, RIP_PORT)).expect("Failed to bind recv socket");

        self.send_socket = Some(send_socket);

        let routing_table = Arc::clone(&self.routing_table);

        thread::spawn(move || {
            let mut buf = [0; 1024];
            loop {
                let (amt, _src) = recv_socket.recv_from(&mut buf).expect("Failed to receive data");
                let received_data = &buf[..amt];
                let mut routing_table = routing_table.lock().unwrap();
                update_routing_table(&mut routing_table, received_data, &config);
            }
        });
    }

    pub fn send_update(&self, dest_ip: Ipv4Addr, exit_interface_ip: Ipv4Addr) {
        let send_socket = self.send_socket.as_ref().expect("Send socket not initialized");
        let dest = SocketAddr::new(IpAddr::V4(dest_ip), RIP_PORT);
    
        let routing_table = self.routing_table.lock().unwrap().clone();
        let data = (exit_interface_ip, routing_table);
        let serialized_data = bincode::serialize(&data).expect("Failed to serialize data");
    
        send_socket.send_to(&serialized_data, &dest).expect("Failed to send data");
    }

    pub fn get_routing_table(&self) -> Vec<Route> {
        let routing_table = self.routing_table.lock().unwrap();
        routing_table.clone()
    }
}

fn update_routing_table(routing_table: &mut Vec<Route>, data: &[u8], config: &[InterfaceWrapper]) {
    let (exit_interface_ip, received_routes): (Ipv4Addr, Vec<Route>) = bincode::deserialize(data).expect("Failed to deserialize data");

    for mut route in received_routes {
        route.next_hop = Some(exit_interface_ip);
        if let Some(exit_interface) = get_exit_interface(config, exit_interface_ip) {
            route.exit_interface = exit_interface;
        }
        add_or_update_route(routing_table, route);
    }
}

fn add_or_update_route(routing_table: &mut Vec<Route>, mut route: Route) {
    let existing_route = routing_table.iter().find(|r| r.network == route.network && r.mask == route.mask);
    if let Some(existing_route) = existing_route {
        if existing_route.metric > route.metric {
            routing_table.retain(|r| r.network != route.network || r.mask != route.mask);
            route.metric += 1;
            routing_table.push(route);
        }
    } else {
        route.metric += 1;
        routing_table.push(route);
    }
}

fn get_network(ip: Ipv4Addr, mask: Ipv4Addr) -> Ipv4Addr {
    let ip_int = u32::from(ip);
    let mask_int = u32::from(mask);
    let network_int = ip_int & mask_int;
    Ipv4Addr::from(network_int)
}

fn mask_to_ip(mask: u8) -> Ipv4Addr {
    let mask_int = !(0xFFFFFFFFu32 >> mask);
    Ipv4Addr::from(mask_int)
}
