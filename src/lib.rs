pub mod config;
pub mod routing;
pub mod client;
pub mod server;


#[cfg(test)]
mod tests {
    use super::config::RouterConfig;
    use super::routing::{RipMessage, Route, RoutingTable};
    use super::client::RipClient;
    use super::server::RipServer;
    use std::thread;
    use std::time::Duration;

    fn load_router_config(filename: &str) -> RouterConfig {
        RouterConfig::from_file(filename)
    }

    #[test]
    fn test_routing_update() {
        let mut table = RoutingTable::new();
        table.update("192.168.1.0".to_string(), "10.1.1.2".to_string(), 1, 24, "192.168.1.1".to_string());
        table.update("192.168.1.0".to_string(), "10.1.1.3".to_string(), 2, 24, "192.168.1.1".to_string());

        let routes = table.get_routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].next_hop, "10.1.1.2");
        assert_eq!(routes[0].metric, 1);
        assert_eq!(routes[0].mask, 24);
        assert_eq!(routes[0].out_interface_ip, "192.168.1.1");
    }

    #[test]
    fn test_rip_message() {
        let routes = vec![
            Route {
                destination: "192.168.1.0".to_string(),
                metric: 1,
                next_hop: "10.1.1.2".to_string(),
                mask: 24,
                out_interface_ip: "192.168.1.1".to_string(),
            },
            Route {
                destination: "10.1.2.0".to_string(),
                metric: 2,
                next_hop: "10.1.1.3".to_string(),
                mask: 24,
                out_interface_ip: "192.168.1.1".to_string(),
            },
        ];

        let message = RipMessage::new("10.1.1.1".to_string(), routes.clone());
        assert_eq!(message.sender, "10.1.1.1");
        assert_eq!(message.routes, routes);
    }

    #[test]
    fn test_client_server_communication() {
        let client = RipClient::new("127.0.0.1:5000");
        let mut server = RipServer::new("127.0.0.1:5001");

        let routes = vec![
            Route {
                destination: "192.168.1.0".to_string(),
                metric: 1,
                next_hop: "10.1.1.2".to_string(),
                mask: 24,
                out_interface_ip: "192.168.1.1".to_string(),
            },
        ];
        let message = RipMessage::new("10.1.1.1".to_string(), routes);

        thread::spawn(move || {
            client.send_rip_message("127.0.0.1:5001", &message);
        });

        thread::sleep(Duration::from_secs(1));
        server.receive_rip_message();

        let routes = server.routing_table.get_routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].next_hop, "10.1.1.1");
        assert_eq!(routes[0].metric, 2);
        assert_eq!(routes[0].mask, 24);
        assert_eq!(routes[0].out_interface_ip, "192.168.1.1");
    }

    #[test]
    fn test_routing_with_real_yaml() {
        let r1_config = load_router_config("./config/routeur-r1.yaml");
        let r2_config = load_router_config("./config/routeur-r2.yaml");
        let r3_config = load_router_config("./config/routeur-r3.yaml");

        // Initialisation des tables de routage
        let mut r1_table = RoutingTable::new();
        let mut r2_table = RoutingTable::new();
        let mut r3_table = RoutingTable::new();

        // R1 initialise ses routes
        for interface in r1_config.interface {
            r1_table.update(
                interface.ip.clone(), 
                interface.ip.clone(), 
                0, 
                interface.mask, 
                interface.ip.clone()
            );
        }

        // R2 initialise ses routes
        for interface in r2_config.interface {
            r2_table.update(
                interface.ip.clone(), 
                interface.ip.clone(), 
                0, 
                interface.mask, 
                interface.ip.clone()
            );
        }

        // R3 initialise ses routes
        for interface in r3_config.interface {
            r3_table.update(
                interface.ip.clone(), 
                interface.ip.clone(), 
                0, 
                interface.mask, 
                interface.ip.clone()
            );
        }

        // Simuler l'envoi de messages RIP de R2 à R1 et R3
        let r2_routes = r2_table.get_routes();
        let r2_message = RipMessage::new("10.1.1.1".to_string(), r2_routes.clone());

        for route in r2_message.routes {
            r1_table.update(
                route.destination.clone(), 
                r2_message.sender.clone(), 
                route.metric + 1, 
                route.mask, 
                route.out_interface_ip.clone()
            );
            r3_table.update(
                route.destination.clone(), 
                r2_message.sender.clone(), 
                route.metric + 1, 
                route.mask, 
                route.out_interface_ip.clone()
            );
        }

        // Vérifiez que les tables de routage de R1 et R3 sont mises à jour correctement
        let r1_routes = r1_table.get_routes();
        let r3_routes = r3_table.get_routes();

        // Assurez-vous que R1 et R3 ont appris les routes de R2
        assert!(r1_routes.iter().any(|r| r.destination == "10.1.1.1"));
        assert!(r3_routes.iter().any(|r| r.destination == "10.1.1.1"));

        // Assurez-vous que les métriques et next hop sont corrects
        let r1_route_to_r2 = r1_routes.iter().find(|r| r.destination == "10.1.1.1").unwrap();
        assert_eq!(r1_route_to_r2.metric, 1);
        assert_eq!(r1_route_to_r2.next_hop, "10.1.1.1");

        let r3_route_to_r2 = r3_routes.iter().find(|r| r.destination == "10.1.1.1").unwrap();
        assert_eq!(r3_route_to_r2.metric, 1);
        assert_eq!(r3_route_to_r2.next_hop, "10.1.1.1");
    }
}
