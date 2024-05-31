pub mod config;
pub mod routing;
pub mod client;
pub mod server;

#[cfg(test)]
mod tests {
    use super::routing::{RipMessage, Route, RoutingTable};
    use super::client::RipClient;
    use super::server::RipServer;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_routing_update() {
        let mut table = RoutingTable::new();
        table.update("192.168.1.0".to_string(), "10.1.1.2".to_string(), 1);
        table.update("192.168.1.0".to_string(), "10.1.1.3".to_string(), 2);

        let routes = table.get_routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].next_hop, "10.1.1.2");
        assert_eq!(routes[0].metric, 1);
    }

    #[test]
    fn test_rip_message() {
        let routes = vec![
            Route {
                destination: "192.168.1.0".to_string(),
                metric: 1,
                next_hop: "10.1.1.2".to_string(),
            },
            Route {
                destination: "10.1.2.0".to_string(),
                metric: 2,
                next_hop: "10.1.1.3".to_string(),
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
            },
        ];
        let message = RipMessage::new("10.1.1.1".to_string(), routes);

        thread::spawn(move || {
            client.send_rip_message("127.0.0.1:5001", &message);
        });

        thread::sleep(Duration::from_secs(1));
        server.receive_rip_message();

        let routes = server.routing_table.get_routes();
        println!("{:?}", routes);
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].next_hop, "10.1.1.1");
        assert_eq!(routes[0].metric, 2);
    }
}
