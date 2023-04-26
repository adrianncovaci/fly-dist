use std::{collections::HashMap, io::Write};

use ::serde::{Deserialize, Serialize};
use anyhow::Context;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum BroadcastPayload {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

impl fly_dist::Payload for BroadcastPayload {}

struct BroadcastState {
    id: usize,
    messages: Vec<usize>,
}

impl fly_dist::Node<BroadcastPayload> for BroadcastState {
    fn handle(
        &mut self,
        msg: fly_dist::Message<BroadcastPayload>,
        output: &mut std::io::StdoutLock,
    ) -> anyhow::Result<()> {
        let mut response = msg.into_response(self.id);
        match response.body.type_ {
            BroadcastPayload::Topology { .. } => {
                response.body.type_ = BroadcastPayload::TopologyOk;

                serde_json::to_writer(&mut *output, &response)
                    .context("serializing into io stream")?;
                output.write_all(b"\n").context("writing new line")?;
            }
            BroadcastPayload::Broadcast { message } => {
                self.messages.push(message);
                response.body.type_ = BroadcastPayload::BroadcastOk;

                serde_json::to_writer(&mut *output, &response)
                    .context("serializing into io stream")?;
                output.write_all(b"\n").context("writing new line")?;
            }
            BroadcastPayload::Read => {
                response.body.type_ = BroadcastPayload::ReadOk {
                    messages: self.messages.clone(),
                };

                serde_json::to_writer(&mut *output, &response)
                    .context("serializing into io stream")?;
                output.write_all(b"\n").context("writing new line")?;
            }
            _ => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let node = BroadcastState {
        id: 0,
        messages: vec![],
    };
    fly_dist::serve(node)
}
