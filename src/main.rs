mod artifacts;
mod contracts;
mod install;
mod messages;
mod paths;
mod store;
mod verify;
use anyhow::{Result, ensure};
use clap::{Parser, Subcommand};
use hacp::v2::Session;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf};
use store::*;

#[derive(Parser)]
#[command(
    version,
    about = "Bilateral coding-agent agreements in a shared directory"
)]
struct Cli {
    #[arg(long, global=true, env="HACP_PEER", value_parser=["a", "b"])]
    peer: Option<String>,
    #[arg(long, global = true, default_value = ".")]
    project: PathBuf,
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Start {
        task: String,
        #[arg(long, num_args=1..)]
        owns: Vec<String>,
    },
    Join {
        task: String,
        #[arg(long, num_args=1..)]
        owns: Vec<String>,
    },
    Status,
    Install {
        #[arg(long,value_parser=["claude","codex","agy","opencode","all"])]
        cli: Option<String>,
        #[arg(long)]
        home: Option<PathBuf>,
    },
    Doctor {
        #[arg(long,value_parser=["claude","codex","agy","opencode","all"])]
        cli: Option<String>,
        #[arg(long)]
        home: Option<PathBuf>,
    },
    Verify {
        contract_id: String,
        #[arg(long, default_value_t = 300)]
        timeout: u64,
        #[arg(long)]
        retry_interrupted: bool,
    },
    Decline {
        contract_id: String,
        digest: String,
    },
    Submit {
        contract_id: String,
        revision: String,
        #[arg(long, default_value = "completed frozen outputs")]
        claim: String,
    },
    Propose {
        contract_id: Option<String>,
        #[arg(long)]
        terms: PathBuf,
    },
    Accept {
        contract_id: String,
        digest: String,
    },
    Ask {
        text: String,
    },
    Answer {
        message_id: String,
        text: String,
    },
    Poll {
        #[arg(long)]
        all: bool,
    },
    Wait {
        #[arg(long, default_value_t = 180)]
        timeout: u64,
    },
    Close {
        #[arg(long)]
        reason: String,
    },
}
fn main() {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(value) => {
            if cli.json {
                println!("{}", serde_json::to_string(&value).unwrap());
            } else {
                println!("{}", serde_json::to_string_pretty(&value).unwrap());
            }
        }
        Err(e) => {
            if cli.json {
                println!("{}", json!({"error":format!("{e:#}")}));
            } else {
                eprintln!("hacp: {e:#}");
            }
            std::process::exit(1);
        }
    }
}
fn run(cli: &Cli) -> Result<Value> {
    match &cli.command {
        Command::Install { cli, home } => {
            return install::run(cli.as_deref(), home.as_deref(), false);
        }
        Command::Doctor { cli, home } => {
            return install::run(cli.as_deref(), home.as_deref(), true);
        }
        _ => (),
    }
    let peer = cli
        .peer
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("specify --peer a|b (or HACP_PEER)"))?;
    match cli.command {
        Command::Poll { all } => return messages::poll(&cli.project, peer, all, None),
        Command::Wait { timeout } => {
            return messages::poll(&cli.project, peer, false, Some(timeout));
        }
        _ => {}
    }
    if let Command::Submit {
        contract_id,
        revision,
        claim,
    } = &cli.command
    {
        return artifacts::submit(&cli.project, peer, contract_id, revision, claim);
    }
    if let Command::Verify {
        contract_id,
        timeout,
        retry_interrupted,
    } = &cli.command
    {
        return verify::verify(
            &cli.project,
            peer,
            contract_id,
            *timeout,
            *retry_interrupted,
        );
    }
    let st = Store::lock(&cli.project)?;
    if let Command::Start { task, owns } = &cli.command {
        ensure!(peer == "a", "only peer a can start");
        ensure!(
            !st.root.join(".hacp/session.json").exists(),
            "session already exists; inspect status to resume; existing sessions are preserved"
        );
        ensure!(!task.trim().is_empty(), "task must not be empty");
        let owns = paths::list(&st.root, owns)?;
        let mut s = Snapshot {
            format: 1,
            session: Session::open(&id("s"), &urn("a"), &urn("b"))?,
            peers: BTreeMap::from([(
                "a".into(),
                Registration {
                    task: task.clone(),
                    owns: owns.clone(),
                },
            )]),
            processed: Default::default(),
            messages: vec![],
            fetched: Default::default(),
            contracts: Default::default(),
            events: vec![],
        };
        event(
            &mut s,
            peer,
            "start",
            json!({"task":task,"files":owns,"next":"peer b joins"}),
        );
        st.commit(&s)?;
        return Ok(json!(s));
    }
    let mut s = st.load()?;
    if let Command::Join { task, owns } = &cli.command {
        ensure!(peer == "b", "only peer b can join");
        ensure!(!task.trim().is_empty(), "task must not be empty");
        let owns = paths::list(&st.root, owns)?;
        paths::disjoint(&st.root, &owns, &s.peers["a"].owns)?;
        s.session.accept(&urn(peer))?;
        s.peers.insert(
            peer.into(),
            Registration {
                task: task.clone(),
                owns: owns.clone(),
            },
        );
        event(
            &mut s,
            peer,
            "join",
            json!({"task":task,"files":owns,"next":"propose contracts"}),
        );
        st.commit(&s)?;
        return Ok(json!(s));
    }
    authorize(&s, peer)?;
    if messages::ingest(&st, &mut s, peer)? {
        st.commit(&s)?;
    }
    match &cli.command {
        Command::Ask { text } | Command::Answer { text, .. } => {
            messages::active(&s)?;
            let (kind, reply) = match &cli.command {
                Command::Answer { message_id, .. } => {
                    ("hacp.skill.answer", Some(message_id.clone()))
                }
                _ => ("hacp.skill.ask", None),
            };
            let message_id = messages::send(&mut s, peer, kind, json!({"text":text}), reply)?;
            st.commit(&s)?;
            Ok(json!({"message_id":message_id}))
        }
        Command::Propose { contract_id, terms } => {
            let t = contracts::terms(&st.root, terms)?;
            if let Some(cid) = contract_id {
                contracts::counter(&st, &mut s, peer, cid, t)
            } else {
                contracts::propose(&st, &mut s, peer, t)
            }
        }
        Command::Accept {
            contract_id,
            digest,
        } => contracts::accept(&st, &mut s, peer, contract_id, digest),
        Command::Decline {
            contract_id,
            digest,
        } => contracts::decline(&st, &mut s, peer, contract_id, digest),
        Command::Status => Ok(json!(s)),
        Command::Close { reason } => {
            ensure!(!reason.trim().is_empty(), "close requires a reason");
            if s.session.state == hacp::v2::SessionState::Opening {
                s.session.abandon(reason)?;
            } else {
                s.session.close(&urn(peer), reason)?;
            }
            messages::send(
                &mut s,
                peer,
                "session.close",
                json!({"reason":reason}),
                None,
            )?;
            event(&mut s, peer, "close", json!({"reason":reason}));
            st.commit(&s)?;
            Ok(json!(s.session))
        }
        _ => unreachable!(),
    }
}
