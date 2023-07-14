use std::io::stdout;

use dist_sys_challenge::{Init, Node, Payload};
use serde::{Deserialize, Serialize};

fn main() -> std::io::Result<()> {
    dist_sys_challenge::run::<UniqueIdsNode>()
}

struct UniqueIdsNode {
    msg_seq_id: usize,
    node_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RequestMessages {
    Generate {},
}

impl Payload for RequestMessages {}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ResponseMessages {
    GenerateOk { id: String },
}

impl Payload for ResponseMessages {}

impl Node for UniqueIdsNode {
    type Msg = RequestMessages;

    fn new(Init::Init { node_id, node_ids }: dist_sys_challenge::Init) -> Self {
        Self {
            msg_seq_id: 1,
            node_id,
        }
    }

    fn process(&mut self, request: &dist_sys_challenge::Message<Self::Msg>) -> std::io::Result<()> {
        match request.payload() {
            RequestMessages::Generate {} => {
                let id = format!("{}@{}", self.msg_seq_id, self.node_id);
                request.respond(
                    &mut stdout(),
                    Some(&mut self.msg_seq_id),
                    ResponseMessages::GenerateOk { id },
                )?;
            }
        }
        Ok(())
    }
}
