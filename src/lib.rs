use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    io::{stdin, stdout},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<P> {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Body<P>,
}

pub trait Payload {}

impl<P: Payload> Message<P> {
    fn respond_error<W: Write>(
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
        msg_id: Option<&mut usize>,
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
                    *id += 1;
                    mid
                }),
                in_reply_to: self.body.msg_id,
                payload,
            },
        }
        .send(writer)
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

#[derive(Debug, Serialize, Deserialize)]
struct Body<P> {
    msg_id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: P,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Init {
    Init {
        node_id: String,
        node_ids: Vec<String>,
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

    init.respond(&mut stdout(), Some(&mut 0), InitOk::InitOk {})?;

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
