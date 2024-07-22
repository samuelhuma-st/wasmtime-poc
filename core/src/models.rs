use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct Workflow {}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Connection {
    pub from: String,  
    pub to: String,    
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeData {
   pub id: String,
   pub name: String,
   pub node_type: String,
   pub parameters: Option<HashMap<String, Value>>,
   pub next_node: Option<String>,
}

impl NodeData {
    pub fn new(
        id: String,
        name: String,
        node_type: String,
        parameters: Option<HashMap<String, Value>>,
        next_node: Option<String>,
    ) -> NodeData {
        NodeData {
            id,
            name,
            next_node,
            node_type,
            parameters,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkflowData {
    pub name: String,
    pub nodes: Vec<NodeData>,
    // pub connections: HashMap<String, Vec<String>>,
    pub connections: Vec<Connection>,
    pub meta_data: Option<HashMap<String, String>>,
}

impl WorkflowData {
    pub fn new(
        name: String,
        nodes: Vec<NodeData>,
        connections:Vec<Connection>,
        meta_data: Option<HashMap<String, String>>,
    ) -> WorkflowData {
        WorkflowData {
            name,
            connections,
            meta_data,
            nodes,
        }
    }
}
