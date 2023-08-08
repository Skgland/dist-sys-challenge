use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    io::Write,
    io::{stdin, stdout},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[serde(transparent)]
pub struct MsgId(usize);

impl MsgId {
    pub const ONE: Self = MsgId(1);
}

impl Display for MsgId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[serde(transparent)]
pub struct NodeId(String);

impl NodeId {
    pub fn seq_kv() -> Self {
        Self(String::from("seq-kv"))
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<P> {
    src: NodeId,
    #[serde(rename = "dest")]
    dst: NodeId,
    body: Body<P>,
}

pub trait Payload {}

impl<P: Payload> Message<P> {
    pub fn new(src: NodeId, dst: NodeId, msg_id: Option<&mut MsgId>, payload: P) -> Self {
        Message {
            src,
            dst,
            body: Body {
                msg_id: msg_id.map(|id| {
                    let mid = *id;
                    *id = MsgId(id.0 + 1);
                    mid
                }),
                in_reply_to: None,
                payload,
            },
        }
    }

    pub fn respond_error<W: Write>(
        &self,
        writer: &mut W,
        code: ErrorCode,
        text: Option<String>,
    ) -> std::io::Result<()> {
        let code = match code {
            ErrorCode::Custom { code } => code,
            standard => standard.discriminant() as usize,
        };

        self.respond(writer, None, Error::Error { code, text })
    }

    pub fn respond<W: Write, R>(
        &self,
        writer: &mut W,
        msg_id: Option<&mut MsgId>,
        payload: R,
    ) -> std::io::Result<()>
    where
        R: Payload,
        Message<R>: Serialize,
    {
        Message {
            src: self.dst.clone(),
            dst: self.src.clone(),
            body: Body {
                msg_id: msg_id.map(|id| {
                    let mid = *id;
                    *id = MsgId(id.0 + 1);
                    mid
                }),
                in_reply_to: self.body.msg_id,
                payload,
            },
        }
        .send(writer)
    }

    pub fn src(&self) -> &NodeId {
        &self.src
    }

    pub fn id(&self) -> Option<MsgId> {
        self.body.msg_id
    }

    pub fn in_response_to(&self) -> Option<MsgId> {
        self.body.in_reply_to
    }

    pub fn send<W: Write>(self, writer: &mut W) -> std::io::Result<()>
    where
        Self: Serialize,
    {
        serde_json::to_writer(&mut *writer, &self)?;
        writer.write_all(b"\n")
    }

    pub fn payload(&self) -> &P {
        &self.body.payload
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Body<P> {
    msg_id: Option<MsgId>,
    in_reply_to: Option<MsgId>,
    #[serde(flatten)]
    payload: P,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Init {
    Init {
        node_id: NodeId,
        node_ids: Vec<NodeId>,
    },
}

impl Payload for Init {}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InitOk {
    InitOk {},
}

impl Payload for InitOk {}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Error {
    Error { code: usize, text: Option<String> },
}

impl Payload for Error {}

#[non_exhaustive]
#[repr(u16)]
#[derive(Debug, Clone)]
pub enum ErrorCode {
    Timeout = 0,
    NodeNotFound = 1,
    NotSupported = 10,
    TemporarilyUnavailable = 11,
    MalformedRequest = 12,
    Chrash = 13,
    Abort = 14,
    KeyDoesNotExist = 20,
    KeyExistsAlready = 21,
    PreConditionFailed = 22,
    TransactionConflict = 30,

    #[non_exhaustive]
    Custom {
        code: usize,
    }, // userdefined error should be >= 100
}

impl<'de> Deserialize<'de> for ErrorCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = usize::deserialize(deserializer)?;
        Ok(match val {
            0 => Self::Timeout,
            1 => Self::NodeNotFound,
            10 => Self::NotSupported,
            11 => Self::TemporarilyUnavailable,
            12 => Self::MalformedRequest,
            13 => Self::Chrash,
            14 => Self::Abort,
            20 => Self::KeyDoesNotExist,
            21 => Self::KeyExistsAlready,
            22 => Self::PreConditionFailed,
            30 => Self::TransactionConflict,
            _ => Self::custom(val),
        })
    }
}

impl ErrorCode {
    pub fn custom(code: usize) -> Self {
        assert!(code >= 1000);
        Self::Custom { code }
    }

    fn discriminant(&self) -> u16 {
        // SAFETY: Because `Self` is marked `repr(u16)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u16` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u16>() }
    }
}

#[derive(Debug, Deserialize)]
struct EmptyBody {}

impl Payload for EmptyBody {}

pub trait Node {
    type Msg: serde::de::DeserializeOwned + Payload;

    fn new(init: Init) -> Self;

    fn process(&mut self, request: &Message<Self::Msg>) -> std::io::Result<()>;
}

pub fn run<N: Node>() -> std::io::Result<()> {
    let stdin = stdin();

    let init: Message<Init> = {
        let mut init = String::new();
        stdin.read_line(&mut init)?;
        serde_json::from_str(&init)?
    };

    init.respond(&mut stdout(), Some(&mut MsgId(0)), InitOk::InitOk {})?;

    let mut node = N::new(init.body.payload);

    let mut line;

    loop {
        line = String::new();
        stdin.read_line(&mut line)?;
        let msg = serde_json::from_str::<Message<_>>(&line);
        match msg {
            Ok(msg) => {
                if let Err(err) = node.process(&msg) {
                    msg.respond_error(&mut stdout(), ErrorCode::Chrash, Some(err.to_string()))?;
                }
            }
            Err(err) => {
                if let Ok(fb_msg) = serde_json::from_str::<Message<EmptyBody>>(&line) {
                    fb_msg.respond_error(
                        &mut stdout(),
                        ErrorCode::MalformedRequest,
                        Some(format!("{err}")),
                    )?;
                } else {
                    return Err(err.into());
                }
            }
        };
    }
}
