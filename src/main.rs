use std::io;

use clap::{Parser, Subcommand};

mod changeset;
mod element;
mod replication;
mod util;

const DEFAULT_SERVER: &str = "https://www.openstreetmap.org";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CliArgs {
    ///
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

    Replication(replication::CliArgs),
}

fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();

    let server = &args.server;

    let res = match args.subcommand {
        Command::Node(args) => element::run(server, "node", &args),
        Command::Way(args) => element::run(server, "way", &args),
        Command::Relation(args) => element::run(server, "relation", &args),
        Command::Changeset(args) => changeset::run(server, &args),

        Command::Replication(args) => replication::run(&args),
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
