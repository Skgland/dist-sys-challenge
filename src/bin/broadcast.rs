use std::{
    collections::{HashMap, HashSet},
    io::stdout,
};

use dist_sys_challenge::{Init, Node, Payload};
use serde::{Deserialize, Serialize};

fn main() -> std::io::Result<()> {
    dist_sys_challenge::run::<BroadcastNode>()
}

struct BroadcastNode {
    msg_seq_id: usize,
    node_id: String,
    neighbors: Vec<String>,
    seen: HashSet<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RequestMessages {
    Broadcast {
        message: usize,
    },
    Read {},
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
}

impl Payload for RequestMessages {}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ResponseMessages {
    BroadcastOk {},
    ReadOk { messages: Vec<usize> },
    TopologyOk {},
}

impl Payload for ResponseMessages {}

impl Node for BroadcastNode {
    type Msg = RequestMessages;

    fn new(Init::Init { node_id, node_ids }: dist_sys_challenge::Init) -> Self {
        Self {
            msg_seq_id: 1,
            node_id,
            neighbors: node_ids,
            seen: HashSet::new(),
        }
    }

    fn process(&mut self, request: &dist_sys_challenge::Message<Self::Msg>) -> std::io::Result<()> {
        match request.payload() {
            RequestMessages::Broadcast { message } => {
                self.seen.insert(*message);
                request.respond(
                    &mut stdout(),
                    Some(&mut self.msg_seq_id),
                    ResponseMessages::BroadcastOk {},
                )?;
            }
            RequestMessages::Read {} => {
                request.respond(
                    &mut stdout(),
                    Some(&mut self.msg_seq_id),
                    ResponseMessages::ReadOk {
                        messages: self.seen.iter().copied().collect(),
                    },
                )?;
            }
            RequestMessages::Topology { topology } => {
                topology
                    .get(&self.node_id)
                    .cloned()
                    .map(|neighbors| self.neighbors = neighbors);
                request.respond(
                    &mut stdout(),
                    Some(&mut self.msg_seq_id),
                    ResponseMessages::TopologyOk {},
                )?;
            }
        }
        Ok(())
    }
}
