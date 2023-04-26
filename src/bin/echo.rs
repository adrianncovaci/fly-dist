pub mod broadcast;

use std::io::{StdoutLock, Write};

use ::serde::{Deserialize, Serialize};
use anyhow::Context;
use fly_dist::{Message, Node};

struct EchoState {
    id: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum EchoPayload {
    Echo { echo: String },
    EchoOk { echo: String },
}

impl fly_dist::Payload for EchoPayload {}

impl Node<EchoPayload> for EchoState {
    fn handle(&mut self, msg: Message<EchoPayload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let mut response = msg.into_response(self.id);
        match response.body.type_ {
            EchoPayload::Echo { echo } => {
                response.body.type_ = EchoPayload::EchoOk { echo };

                serde_json::to_writer(&mut *output, &response)
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
    let node = EchoState { id: 0 };
    fly_dist::serve(node)
}
