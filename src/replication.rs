use std::error::Error;
use std::fmt::LowerExp;
use std::io::{self, BufRead, Read, Write};

use chrono::{DateTime, FixedOffset, Utc};
use clap::Parser;

const DEFAULT_SERVER: &str = "https://planet.openstreetmap.org"; // FIXME: make this configurable

#[derive(Parser)]
pub struct CliArgs {
    /// Timestamp in RFC 2822 or RFC 3339 format
    #[arg(long)]
    since: String,

    /// Run forever, writing new replication file URLs to stdout as they are published
    #[arg(long)]
    watch: bool,
}

pub fn run(args: &CliArgs) -> Result<(), Box<dyn Error>> {
    // let endpoint = format!("{}/replication/minute/", server);

    let since: DateTime<Utc> = DateTime::parse_from_rfc2822(&args.since)
        .ok()
        .or_else(|| DateTime::parse_from_rfc3339(&args.since).ok())
        .expect("failed to parse --since as timestamp")
        .into();

    eprintln!("binary searching to find starting sequence number");

    let start_seqno = timestamp_to_seqno(since);

    let latest = get_current_state_info();

    for seqno in start_seqno..latest.seqno {
        writeln!(io::stdout(), "{}", osc_url(seqno))?;
    }

    Ok(())
}

fn timestamp_to_seqno(timestamp: DateTime<Utc>) -> u64 {
    let mut upper = get_current_state_info();

    if upper.timestamp < timestamp || upper.seqno == 0 {
        return upper.seqno;
    }

    let mut guess: u64 = 0;
    let mut lower: Option<StateInfo>;

    // find a state file that is below the required timestamp
    loop {
        // eprintln!("Trying with Id {}", guess);
        lower = get_state_info(guess).ok();

        if let Some(lower) = &lower {
            if lower.timestamp >= timestamp {
                return lower.seqno;
            } else {
                break;
            }
        }

        if lower.is_none() {
            let step = (upper.seqno - guess) / 2;
            if step == 0 {
                return upper.seqno;
            }
            guess += step;
        }
    }

    let mut lower = lower.unwrap();

    assert!(lower.seqno < upper.seqno);

    loop {
        // dbg!(&lower);
        // dbg!(&upper);

        let time_delta = (upper.timestamp - lower.timestamp).num_seconds();
        let seqno_delta = upper.seqno - lower.seqno;
        let seqno_rate = (seqno_delta as f64) / (time_delta as f64);
        let desired_time_step = (timestamp - lower.timestamp).num_seconds();
        guess = lower.seqno + f64::ceil((desired_time_step as f64) * seqno_rate) as u64;
        if guess == upper.seqno {
            guess = guess - 1;
        }

        // dbg!(&guess);

        let split = get_state_info(guess);

        // TODO: what if split.is_none() (i.e. guess not found)?
        // we should walk up+down to find a nearby split candidate
        let split = split.unwrap();

        // dbg!(&split);

        if split.timestamp < timestamp {
            // eprintln!("guess was too low");
            lower = split;
        } else {
            // eprintln!("guess was too high");
            upper = split;
        }

        if lower.seqno + 1 >= upper.seqno {
            return lower.seqno;
        }
        // eprintln!("trying again");
    }
}

#[derive(Debug)]
struct StateInfo {
    seqno: u64,
    timestamp: DateTime<Utc>,
}

impl StateInfo {
    fn try_from_reader(reader: impl Read) -> Result<Self, ()> {
        let mut seqno: Option<u64> = None;
        let mut timestamp: Option<DateTime<Utc>> = None;

        for line in io::BufReader::new(reader).lines().flatten() {
            if line.starts_with("#") {
                continue;
            }

            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            let mut split = line.split('=');
            let k = split.next().unwrap().trim();
            let v = split.next().unwrap().trim();

            match k {
                "sequenceNumber" => {
                    seqno = Some(v.parse().unwrap());
                }
                "timestamp" => {
                    timestamp = DateTime::parse_from_rfc3339(&v.replace('\\', ""))
                        .ok()
                        .map(|dt| dt.try_into().ok())
                        .flatten();
                }
                _ => continue,
            }
        }

        Ok(StateInfo {
            seqno: seqno.expect("seqno not found"),
            timestamp: timestamp.expect("timestamp not found"),
        })
    }
}

fn get_current_state_info() -> StateInfo {
    let res = ureq::get(&latest_state_url()).call().unwrap();

    StateInfo::try_from_reader(res.into_reader()).unwrap()
}

fn get_state_info(seqno: u64) -> Result<StateInfo, ureq::Error> {
    // TODO
    eprintln!("getting state for seqno {}", seqno);
    let res = ureq::get(&state_url(seqno)).call()?;

    Ok(StateInfo::try_from_reader(res.into_reader()).unwrap())
}

fn latest_state_url() -> String {
    format!("{}/replication/minute/state.txt", DEFAULT_SERVER)
}

fn state_url(seqno: u64) -> String {
    let triplet = seqno_to_triplet(seqno);
    format!(
        "{}/replication/minute/{:03}/{:03}/{:03}.state.txt",
        DEFAULT_SERVER, triplet.0, triplet.1, triplet.2
    )
}

fn osc_url(seqno: u64) -> String {
    let triplet = seqno_to_triplet(seqno);
    format!(
        "{}/replication/minute/{:03}/{:03}/{:03}.osc.gz",
        DEFAULT_SERVER, triplet.0, triplet.1, triplet.2
    )
}

fn seqno_to_triplet(seqno: u64) -> (u16, u16, u16) {
    let hi = (seqno / 1_000_000) as u16;
    let md = ((seqno % 1_000_000) / 1000) as u16;
    let lo = (seqno % 1000) as u16;

    (hi, md, lo)
}
