use std::{
    collections::HashMap,
    io::stdout,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use dist_sys_challenge::{ErrorCode, Init, Message, MsgId, Node, NodeId, Payload};
use serde::{Deserialize, Serialize};
use serde_json::json;

fn main() -> std::io::Result<()> {
    dist_sys_challenge::run::<GrowOnlyNode>()
}

struct GrowOnlyNode {
    processor: Sender<Action>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RequestMessages {
    Add { delta: usize },
    Read {},
    ReadOk { value: serde_json::Value },
    CasOk {},
    Error { code: ErrorCode, text: String },
}

impl Payload for RequestMessages {}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ResponseMessages {
    AddOk {},
    ReadOk { value: usize },
}

impl Payload for ResponseMessages {}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SeqKVRequest {
    Read {
        key: serde_json::Value,
    },
    Cas {
        key: serde_json::Value,
        from: serde_json::Value,
        to: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        create_if_not_exists: Option<bool>,
    },
}
impl Payload for SeqKVRequest {}

enum Action {
    Commit,
    Msg(Message<RequestMessages>),
}

enum Pending {
    Write { delta: usize, target: usize },
}

fn processor(
    Init::Init {
        node_id,
        node_ids: _,
    }: Init,
    channel: Receiver<Action>,
) -> std::io::Result<()> {
    let mut msg_seq_id = MsgId::ONE;
    let mut cur_delta = 0;
    let mut last_read = 0;
    let mut pending = HashMap::<MsgId, Pending>::new();

    for action in channel {
        match action {
            Action::Commit => {
                Message::new(
                    node_id.clone(),
                    NodeId::seq_kv(),
                    Some(&mut msg_seq_id),
                    SeqKVRequest::Read {
                        key: serde_json::Value::String("counter".to_owned()),
                    },
                )
                .send(&mut stdout())?;
            }
            Action::Msg(in_msg) => match in_msg.payload() {
                RequestMessages::Add { delta } => {
                    cur_delta += delta;
                    in_msg.respond(
                        &mut stdout(),
                        Some(&mut msg_seq_id),
                        ResponseMessages::AddOk {},
                    )?;
                }
                RequestMessages::Read {} => {
                    in_msg.respond(
                        &mut stdout(),
                        Some(&mut msg_seq_id),
                        ResponseMessages::ReadOk { value: last_read },
                    )?;
                }
                RequestMessages::ReadOk { value } => {
                    let value = value.as_u64().unwrap() as usize;
                    last_read = last_read.max(value);

                    if cur_delta != 0 {
                        let old = last_read;
                        let new = last_read + cur_delta;

                        let out_msg = Message::new(
                            node_id.clone(),
                            NodeId::seq_kv(),
                            Some(&mut msg_seq_id),
                            SeqKVRequest::Cas {
                                key: serde_json::Value::String("counter".to_owned()),
                                from: json!(old),
                                to: json!(new),
                                create_if_not_exists: (old == 0).then_some(true),
                            },
                        );

                        pending.insert(
                            out_msg.id().unwrap(),
                            Pending::Write {
                                delta: cur_delta,
                                target: new,
                            },
                        );

                        cur_delta = 0;

                        out_msg.send(&mut stdout())?;
                    }
                }
                RequestMessages::CasOk {} => {
                    if let Some(Pending::Write { delta: _, target }) =
                        pending.remove(&in_msg.in_response_to().unwrap())
                    {
                        last_read = last_read.max(target)
                    }
                }
                RequestMessages::Error { code, text: _ } => match code {
                    ErrorCode::PreConditionFailed => {
                        if let Some(Pending::Write { delta, target: _ }) =
                            pending.remove(&in_msg.in_response_to().unwrap())
                        {
                            cur_delta += delta;

                            // cas failed return delta to cur_delta and read current value
                            Message::new(
                                node_id.clone(),
                                NodeId::seq_kv(),
                                Some(&mut msg_seq_id),
                                SeqKVRequest::Read {
                                    key: serde_json::Value::String("counter".to_owned()),
                                },
                            )
                            .send(&mut stdout())?;
                        }
                    }
                    ErrorCode::KeyDoesNotExist => {
                        // counter not yet initialized
                        let out_msg = Message::new(
                            node_id.clone(),
                            NodeId::seq_kv(),
                            Some(&mut msg_seq_id),
                            SeqKVRequest::Cas {
                                key: serde_json::Value::String("counter".to_owned()),
                                from: json!(0),
                                to: json!(cur_delta),
                                create_if_not_exists: Some(true),
                            },
                        );

                        pending.insert(
                            out_msg.id().unwrap(),
                            Pending::Write {
                                delta: cur_delta,
                                target: cur_delta,
                            },
                        );

                        cur_delta = 0;

                        out_msg.send(&mut stdout())?;
                    }
                    _ => unimplemented!("Unexpected Error"),
                },
            },
        }
    }

    Ok(())
}

impl Node for GrowOnlyNode {
    type Msg = RequestMessages;

    fn new(init: dist_sys_challenge::Init) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();

        let sender_clone = sender.clone();
        std::thread::spawn(|| processor(init, receiver));
        std::thread::spawn(move || -> std::io::Result<()> {
            loop {
                std::thread::sleep(Duration::from_millis(50));
                sender_clone.send(Action::Commit).map_err(
                    |err: std::sync::mpsc::SendError<Action>| {
                        std::io::Error::new(std::io::ErrorKind::BrokenPipe, err)
                    },
                )?;
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
