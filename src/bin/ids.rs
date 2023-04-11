use std::io::{StdoutLock, Write};

use ::serde::Serialize;
use anyhow::Context;
use fly_dist::{Body, Message, Node};
use serde::Deserialize;
use uuid::Uuid;

struct IdsState {
    id: usize,
}

impl fly_dist::Payload for IdsPayload {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum IdsPayload {
    Generate,
    GenerateOk { id: String },
}

impl Node<IdsPayload> for IdsState {
    fn handle(&mut self, msg: Message<IdsPayload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match msg.body.type_ {
            IdsPayload::Generate => {
                let id = IdsState::get_next_id();
                let msg = Message {
                    body: Body {
                        type_: IdsPayload::GenerateOk { id: id.to_string() },
                        msg_id: Some(self.id),
                        in_reply_to: msg.body.msg_id,
                    },
                    src: msg.dest,
                    dest: msg.src,
                };

                serde_json::to_writer(&mut *output, &msg).context("serializing into io stream")?;
                output.write_all(b"\n").context("writing new line")?;
            }
            _ => {}
        }

        self.id += 1;

        Ok(())
    }
}

impl IdsState {
    fn get_next_id() -> Uuid {
        Uuid::new_v4()
    }
}

fn main() -> anyhow::Result<()> {
    let node = IdsState { id: 0 };
    fly_dist::serve(node)
}
