use std::io::{StdoutLock, Write};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

#[derive(Debug, Serialize, Deserialize)]
struct Body {
    #[serde(flatten)]
    type_: MessageType,
    msg_id: Option<usize>,
    in_reply_to: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum MessageType {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
}

struct EchoState {
    id: usize,
}

impl EchoState {
    fn handle(&mut self, msg: Message, output: &mut StdoutLock) -> anyhow::Result<()> {
        match msg.body.type_ {
            MessageType::Init { .. } => {
                let reply = Message {
                    src: msg.dest,
                    dest: msg.src,
                    body: Body {
                        type_: MessageType::InitOk,
                        msg_id: Some(self.id),
                        in_reply_to: msg.body.msg_id.into(),
                    },
                };

                serde_json::to_writer(&mut *output, &reply)
                    .context("serializing into io stream")?;
                output.write_all(b"\n").context("writing new line")?;
            }
            MessageType::Echo { echo } => {
                let reply = Message {
                    src: msg.dest,
                    dest: msg.src,
                    body: Body {
                        type_: MessageType::EchoOk { echo },
                        msg_id: Some(self.id),
                        in_reply_to: msg.body.msg_id.into(),
                    },
                };

                serde_json::to_writer(&mut *output, &reply)
                    .context("serializing into io stream")?;
                output.write_all(b"\n").context("writing new line")?;
            }
            MessageType::InitOk => {}
            MessageType::EchoOk { .. } => {}
        }

        self.id += 1;

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let mut stdout = std::io::stdout().lock();
    let stdin = std::io::stdin().lock();

    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    let mut echo = EchoState { id: 0 };

    for input in inputs {
        let input = input?;
        echo.handle(input, &mut stdout).context("handling input")?;
    }

    Ok(())
}
