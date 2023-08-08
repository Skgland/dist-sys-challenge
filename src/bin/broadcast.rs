use std::{
    collections::{HashMap, HashSet},
    io::stdout,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use dist_sys_challenge::{Init, Message, MsgId, Node, NodeId, Payload};
use serde::{Deserialize, Serialize};

fn main() -> std::io::Result<()> {
    dist_sys_challenge::run::<BroadcastNode>()
}

struct BroadcastNode {
    processor: Sender<Action>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RequestMessages {
    Broadcast {
        message: usize,
    },
    Read {},
    Topology {
        topology: HashMap<NodeId, Vec<NodeId>>,
    },
    Gossip {
        news: Vec<News>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
enum News {
    // sender knows but belives we don't know
    NewValue(usize),
    // sender knwows we know, but we have send them recently (we didn't knew they knew)
    VerifiedValue(usize),
}

#[derive(Debug, PartialEq, Eq)]
enum Knowledge {
    // We know they know (received from them), but they don't know we know
    ToBeConfirmed,
    // We know they know and they know we know
    Confirmed,
}

impl Payload for RequestMessages {}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ResponseMessages {
    BroadcastOk {},
    ReadOk { messages: Vec<usize> },
    TopologyOk {},
    // See RequestMessages
    Gossip { news: Vec<News> },
}

impl Payload for ResponseMessages {}

enum Action {
    Msg(Message<RequestMessages>),
    Gossip,
}

fn processor(
    Init::Init {
        node_id,
        node_ids: mut neighbors,
    }: Init,
    channel: Receiver<Action>,
) -> std::io::Result<()> {
    let mut msg_seq_id = MsgId::ONE;
    let mut seen = HashSet::<usize>::new();
    let mut knowledge = HashMap::<(NodeId, usize), Knowledge>::new();

    for action in channel {
        match action {
            Action::Msg(request) => match request.payload() {
                RequestMessages::Broadcast { message } => {
                    seen.insert(*message);
                    request.respond(
                        &mut stdout(),
                        Some(&mut msg_seq_id),
                        ResponseMessages::BroadcastOk {},
                    )?;
                }
                RequestMessages::Read {} => {
                    request.respond(
                        &mut stdout(),
                        Some(&mut msg_seq_id),
                        ResponseMessages::ReadOk {
                            messages: seen.iter().copied().collect(),
                        },
                    )?;
                }
                RequestMessages::Topology { topology } => {
                    topology
                        .get(&node_id)
                        .cloned()
                        .map(|new_neighbors| neighbors = new_neighbors);
                    request.respond(
                        &mut stdout(),
                        Some(&mut msg_seq_id),
                        ResponseMessages::TopologyOk {},
                    )?;
                }
                RequestMessages::Gossip { news } => {
                    news.iter().for_each(|news| {
                        let (val, kind) = match news {
                            News::NewValue(val) => (*val, Knowledge::ToBeConfirmed),
                            News::VerifiedValue(val) => (*val, Knowledge::Confirmed),
                        };
                        seen.insert(val);
                        knowledge.insert((request.src().to_owned(), val), kind);
                    });
                }
            },
            Action::Gossip => {
                for neighbor in &neighbors {
                    let news = seen
                        .iter()
                        .copied()
                        .filter_map(|value| match knowledge.get(&(neighbor.clone(), value)) {
                            Some(Knowledge::ToBeConfirmed) => Some(News::VerifiedValue(value)),
                            Some(Knowledge::Confirmed) => None,
                            None => Some(News::NewValue(value)),
                        })
                        .collect::<Vec<_>>();

                    if !news.is_empty() {
                        Message::new(
                            node_id.clone(),
                            neighbor.to_owned(),
                            Some(&mut msg_seq_id),
                            ResponseMessages::Gossip { news },
                        )
                        .send(&mut stdout())?;
                    }
                }
            }
        }
    }

    Ok(())
}

impl Node for BroadcastNode {
    type Msg = RequestMessages;

    fn new(init: dist_sys_challenge::Init) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();

        let sender_clone = sender.clone();
        std::thread::spawn(|| processor(init, receiver));
        std::thread::spawn(move || -> std::io::Result<()> {
            loop {
                std::thread::sleep(Duration::from_millis(50));
                sender_clone
                    .send(Action::Gossip)
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::BrokenPipe, err))?;
            }
        });

        Self { processor: sender }
    }

    fn process(&mut self, request: &dist_sys_challenge::Message<Self::Msg>) -> std::io::Result<()> {
        self.processor
            .send(Action::Msg(request.clone()))
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::BrokenPipe, err))?;
        Ok(())
    }
}
