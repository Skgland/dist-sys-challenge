use std::io::stdout;

use dist_sys_challenge::{Node, Payload};
use serde::{Deserialize, Serialize};

fn main() -> std::io::Result<()> {
    dist_sys_challenge::run::<EchoNode>()
}

struct EchoNode {
    msg_seq_id: usize,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum EchoMessages {
    Echo { echo: String },
}

impl Payload for EchoMessages {}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum EchoOkMessage {
    EchoOk { echo: String },
}

impl Payload for EchoOkMessage {}

impl Node for EchoNode {
    type Msg = EchoMessages;

    fn new(_: dist_sys_challenge::Init) -> Self {
        Self { msg_seq_id: 1 }
    }

    fn process(&mut self, request: &dist_sys_challenge::Message<Self::Msg>) -> std::io::Result<()> {
        match request.payload() {
            EchoMessages::Echo { echo } => {
                request.respond(
                    &mut stdout(),
                    Some(&mut self.msg_seq_id),
                    EchoOkMessage::EchoOk { echo: echo.clone() },
                )?;
            }
        }
        Ok(())
    }
}

#[test]
fn deserialize() {
    use dist_sys_challenge::Message;
    serde_json::from_str::<Message<EchoMessages>>("{\"id\":90,\"src\":\"c2\",\"dest\":\"n0\",\"body\":{\"echo\":\"Please echo 98\",\"type\":\"echo\",\"msg_id\":45}}\n").expect("should be a valid message!");
    serde_json::from_str::<Message<EchoMessages>>(
        r#"{"id":2,"src":"c2","dest":"n0","body":{"echo":"Please echo 15","type":"echo","msg_id":1}}"#,
    ).expect("should be a valid message");
}
