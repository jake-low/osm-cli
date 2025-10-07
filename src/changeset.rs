use std::io::{self, Write};

use clap::Parser;

use crate::util::{Format, content_type};

#[derive(Parser)]
pub struct CliArgs {
    /// Output format to use
    #[arg(short, long, default_value_t = Format::Xml)]
    format: Format,

    /// Fetch the full diff instead of the metadata
    #[arg(long)]
    diff: bool,

    /// Changeset ID to retrieve
    id: u64,
}

pub fn run(client: &ureq::Agent, server: &str, args: &CliArgs) -> anyhow::Result<()> {
    let endpoint = if args.diff {
        format!("{}/api/0.6/changeset/{}/download", server, args.id)
    } else {
        format!("{}/api/0.6/changeset/{}", server, args.id)
    };

    let req = client.get(&endpoint).header("Accept", args.format.mimetype());
    let res = req.call()?;

    match content_type(&res) {
        Some("application/json") => {
            jsonxf::pretty_print_stream(&mut res.into_body().into_reader(), &mut io::stdout())?;
            writeln!(&mut io::stdout())?; // add trailing newline
        }
        _ => {
            io::copy(&mut res.into_body().into_reader(), &mut io::stdout())?;
        }
    }

    Ok(())
}
