use std::io;
use std::time::Duration;

use clap::{Parser, Subcommand};

mod changeset;
mod element;
mod replication;
mod util;

const DEFAULT_SERVER: &str = "https://www.openstreetmap.org";
const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CliArgs {
    #[arg(long, default_value = DEFAULT_SERVER)]
    server: String,

    #[command(subcommand)]
    subcommand: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Get info about a Node
    Node(element::CliArgs),
    /// Get info about a Way
    Way(element::CliArgs),
    /// Get info about a Relation
    Relation(element::CliArgs),
    /// Get info about a Changeset
    Changeset(changeset::CliArgs),
    /// Subscribe to info about new OSM edits or changesets
    Replication(replication::CliArgs),
}

fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    let server = &args.server;

    let config = ureq::config::Config::builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .user_agent(USER_AGENT)
        .build();
    let client = ureq::Agent::new_with_config(config);

    let res = match args.subcommand {
        Command::Node(args) => element::run(&client, server, "node", &args),
        Command::Way(args) => element::run(&client, server, "way", &args),
        Command::Relation(args) => element::run(&client, server, "relation", &args),
        Command::Changeset(args) => changeset::run(&client, server, &args),
        Command::Replication(args) => replication::run(&client, &args),
    };

    match res {
        Err(error) => {
            if let Some(io_error) = error.downcast_ref::<io::Error>() {
                if io_error.kind() == io::ErrorKind::BrokenPipe {
                    Ok(())
                } else {
                    Err(error)
                }
            } else {
                Err(error)
            }
        }
        Ok(_) => Ok(()),
    }
}
