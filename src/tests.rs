use prettytable::{Table, Row, Cell, row, cell};

use crate::router::InterfaceWrapper;
use core::net::Ipv4Addr;
use crate::rip::Router;
use std::time::Duration;
use crate::rip::Route;

fn print_routing_table(router_name: &str, routing_table: &[Route]) {
    let mut table = Table::new();
    table.add_row(row![cell!("Network"), cell!("Mask"), cell!("Next Hop"), cell!("Metric")]);

    for route in routing_table {

        let network_cell = Cell::new(&route.network.to_string());
        let mask_cell = Cell::new(&route.mask.to_string());
        let next_hop_cell = Cell::new(&route.next_hop.to_string());
        let metric_cell = Cell::new(&route.metric.to_string());

        let row = Row::new(vec![network_cell, mask_cell, next_hop_cell, metric_cell]);

        table.add_row(row);
    }

    println!("{} routing table:", router_name);
    table.printstd();
}

#[test]
fn test_rip_simulation() {
    let router1_config = InterfaceWrapper::from_yaml("./config/routeur-r1.yaml");
    let router2_config = InterfaceWrapper::from_yaml("./config/routeur-r2.yaml");

    let (tx1, rx1) = std::sync::mpsc::channel();
    let (tx2, rx2) = std::sync::mpsc::channel();

    let ip1 = Ipv4Addr::new(127, 0, 0, 1);
    let handle1 = std::thread::spawn(move || {
        let mut router1 = Router::new(router1_config);
        router1.start(ip1);
        tx1.send(router1).unwrap();
    });

    let ip2 = Ipv4Addr::new(127, 0, 0, 2);
    let handle2 = std::thread::spawn(move || {
        let mut router2 = Router::new(router2_config);
        router2.start(ip2);
        tx2.send(router2).unwrap();
    });

    let mut router1 = rx1.recv().unwrap();
    let mut router2 = rx2.recv().unwrap();

    router1.send_update(ip2);
    router2.send_update(ip1);

    std::thread::sleep(Duration::from_secs(2));

    print_routing_table("Router1", &router1.get_routing_table());
    print_routing_table("Router2", &router2.get_routing_table());

    handle1.join().expect("Router 1 failed");
    handle2.join().expect("Router 2 failed");
}