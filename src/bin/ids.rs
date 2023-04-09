use std::io::{StdoutLock, Write};

use anyhow::Context;
use fly_dist::{Body, Message};
use uuid::Uuid;

struct IdsState {
    id: usize,
}

impl IdsState {
    fn get_next_id() -> Uuid {
        Uuid::new_v4()
    }

    fn handle(&mut self, msg: Message, output: &mut StdoutLock) -> anyhow::Result<()> {
        match msg.body.type_ {
            fly_dist::MessageType::Init { .. } => {
                let msg = Message {
                    src: msg.dest,
                    dest: msg.src,
                    body: Body {
                        type_: fly_dist::MessageType::InitOk,
                        msg_id: Some(self.id),
                        in_reply_to: msg.body.msg_id,
                    },
                };

                serde_json::to_writer(&mut *output, &msg).context("serializing into io stream")?;
                output.write_all(b"\n").context("writing new line")?;
            }
            fly_dist::MessageType::Generate => {
                let id = IdsState::get_next_id();
                let msg = Message {
                    body: Body {
                        type_: fly_dist::MessageType::GenerateOk { id: id.to_string() },
                        msg_id: Some(self.id),
                        in_reply_to: msg.body.msg_id,
                    },
                    src: msg.dest,
                    dest: msg.src,
                };

                eprint!("{:?}: ", msg);

                serde_json::to_writer(&mut *output, &msg).context("serializing into io stream")?;
                output.write_all(b"\n").context("writing new line")?;
            }
            _ => {}
        }

        self.id += 1;

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    //let reply = "{\"src\":\"ids\",\"dest\":\"init\",\"body\": {\"type\": \"generate_ok\", \"in_reply_to\": 2, \"id\": \"00000000-0000-0000-0000-000000000000\", \"msg_id\":2 }}";
    //println!("{:?}", serde_json::from_str::<Message>(reply)?);

    ////let msg: Message = serde_json::from_str(reply).context("parsing message")?;
    //let msg = Message {
    //    src: "ids".to_string(),
    //    dest: "init".to_string(),
    //    body: Body {
    //        type_: fly_dist::MessageType::GenerateOk {
    //            id: "00000000-0000-0000-0000-000000000000".to_string(),
    //        },
    //        msg_id: Some(2),
    //        in_reply_to: Some(2),
    //    },
    //};
    //

    let mut stdout = std::io::stdout().lock();
    let stdin = std::io::stdin().lock();

    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();
    let mut state = IdsState { id: 0 };

    for input in inputs {
        let input = input?;
        state.handle(input, &mut stdout).context("handling input")?;
    }

    Ok(())
}
