use std::io::{BufRead, Lines, StdinLock, StdoutLock, Write};

use anyhow::Context;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    pub dest: String,
    pub body: Body<Payload>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Body<Payload> {
    #[serde(flatten)]
    pub type_: Payload,
    pub msg_id: Option<usize>,
    pub in_reply_to: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Init {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
}

pub trait Node<Payload> {
    fn handle(&mut self, msg: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()>;
}

pub trait Payload {
    fn handle_init(stdin: &mut Lines<StdinLock>, stdout: &mut StdoutLock) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        let init_msg: Message<Init> = serde_json::from_str(
            &stdin
                .next()
                .expect("expecting init message")
                .context("reading init message")?,
        )?;

        let reply = Message {
            src: init_msg.dest,
            dest: init_msg.src,
            body: Body {
                type_: Init::InitOk,
                msg_id: init_msg.body.msg_id,
                in_reply_to: init_msg.body.msg_id.into(),
            },
        };

        serde_json::to_writer(&mut *stdout, &reply).context("serializing into io stream")?;
        stdout.write_all(b"\n").context("writing new line")?;
        Ok(())
    }
}

impl Payload for Init {}

pub fn serve<P>(node: impl Node<P>) -> anyhow::Result<()>
where
    P: Payload + DeserializeOwned,
{
    let mut stdout = std::io::stdout().lock();
    let stdin = std::io::stdin().lock();

    let mut stdin = stdin.lines();

    P::handle_init(&mut stdin, &mut stdout)?;

    let mut node = node;

    for msg in stdin {
        let msg: Message<P> = serde_json::from_str(&msg.context("reading message")?)?;
        node.handle(msg, &mut stdout).context("handling message")?;
    }

    Ok(())
}
