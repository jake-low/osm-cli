use std::error::Error;
use std::io::{self, Write};

use clap::Parser;

use crate::util::Format;

#[derive(Parser)]
pub struct CliArgs {
    /// Output format to use
    #[arg(short, long, default_value_t = Format::Xml)]
    format: Format,

    /// Fetch the full history of the element
    #[arg(long)]
    history: bool,

    /// Element ID to retrieve
    id: u64,
}

pub fn run(server: &str, element_type: &str, args: &CliArgs) -> Result<(), Box<dyn Error>> {
    let endpoint = if args.history {
        format!("{}/api/0.6/{}/{}/history", server, element_type, args.id)
    } else {
        format!("{}/api/0.6/{}/{}", server, element_type, args.id)
    };

    let req = ureq::get(&endpoint).set("Accept", args.format.mimetype());
    let res = req.call()?;

    match res.content_type() {
        "application/json" => {
            jsonxf::pretty_print_stream(&mut res.into_reader(), &mut io::stdout())?;
            writeln!(&mut io::stdout())?; // add trailing newline
        }
        _ => {
            io::copy(&mut res.into_reader(), &mut io::stdout())?;
        }
    }

    Ok(())
}
