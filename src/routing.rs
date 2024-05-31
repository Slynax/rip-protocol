use serde_derive::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Route {
    pub destination: String,
    pub metric: u8,
    pub next_hop: String,
}

#[derive(Debug)]
pub struct RoutingTable {
    pub routes: HashMap<String, Route>,
}

impl RoutingTable {
    pub fn new() -> Self {
        RoutingTable {
            routes: HashMap::new(),
        }
    }

    pub fn update(&mut self, destination: String, next_hop: String, metric: u8) {
        if let Some(route) = self.routes.get(&destination) {
            if route.metric > metric {
                self.routes.insert(destination.clone(), Route { destination, next_hop, metric });
            }
        } else {
            self.routes.insert(destination.clone(), Route { destination, next_hop, metric });
        }
    }

    pub fn get_routes(&self) -> Vec<Route> {
        self.routes.values().cloned().collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RipMessage {
    pub sender: String,
    pub routes: Vec<Route>,
}

impl RipMessage {
    pub fn new(sender: String, routes: Vec<Route>) -> Self {
        RipMessage { sender, routes }
    }
}
