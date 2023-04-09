use std::io::{StdoutLock, Write};

use anyhow::Context;
use fly_dist::{Body, Message, MessageType};

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
            _ => {}
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
