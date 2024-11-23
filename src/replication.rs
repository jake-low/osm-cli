use std::io::{self, BufRead, Read, Write};
use std::thread::sleep;
use std::time::Duration;

use chrono::{DateTime, Utc};
use clap::Parser;

#[derive(Parser)]
pub struct CliArgs {
    /// Timestamp in RFC 2822 or RFC 3339 format
    #[arg(long)]
    since: Option<String>,

    #[arg(long)]
    seqno: Option<u64>,

    /// Run forever, writing new replication file URLs to stdout as they are published
    #[arg(long)]
    watch: bool,

    /// Server short name [minute, changesets] or URL
    server: String,
}

pub fn run(args: &CliArgs) -> anyhow::Result<()> {
    let server = match &args.server[..] {
        "minute" => "https://planet.openstreetmap.org/replication/minute",
        "changesets" => "https://planet.openstreetmap.org/replication/changesets",
        url => url,
    };

    let start_seqno = if let Some(since) = &args.since {
        let since: DateTime<Utc> = DateTime::parse_from_rfc2822(since)
            .ok()
            .or_else(|| DateTime::parse_from_rfc3339(since).ok())
            .expect("failed to parse --since as timestamp")
            .into();
        eprintln!("binary searching to find starting sequence number");

        timestamp_to_seqno(server, since)?
    } else if let Some(seqno) = args.seqno {
        seqno
    } else {
        panic!("require either --since or --seqno");
    };

    let latest = get_current_state_info(server)?;

    for seqno in start_seqno..=latest.seqno {
        writeln!(io::stdout(), "{}", osc_url(server, seqno))?;
    }

    if args.watch {
        let mut seqno = latest.seqno;

        loop {
            sleep(Duration::from_secs(60));
            let latest = get_current_state_info(server)?;
            while seqno < latest.seqno {
                seqno += 1;
                writeln!(io::stdout(), "{}", osc_url(server, seqno))?;
            }
        }
    }

    Ok(())
}

fn timestamp_to_seqno(server: &str, timestamp: DateTime<Utc>) -> anyhow::Result<u64> {
    let mut upper = get_current_state_info(server)?;

    if upper.timestamp < timestamp || upper.seqno == 0 {
        return Ok(upper.seqno);
    }

    let mut guess: u64 = 0;
    let mut lower: Option<StateInfo>;

    // find a state file that is below the required timestamp
    loop {
        // eprintln!("Trying with Id {}", guess);
        lower = get_state_info(server, guess).ok();

        if let Some(lower) = &lower {
            if lower.timestamp >= timestamp {
                return Ok(lower.seqno);
            } else {
                break;
            }
        }

        if lower.is_none() {
            let step = (upper.seqno - guess) / 2;
            if step == 0 {
                return Ok(upper.seqno);
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
            guess -= 1;
        }

        // dbg!(&guess);

        let split = get_state_info(server, guess);

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
            return Ok(lower.seqno);
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
    fn try_from_reader(reader: impl Read) -> anyhow::Result<Self> {
        let mut seqno: Option<u64> = None;
        let mut timestamp: Option<DateTime<Utc>> = None;

        for line in io::BufReader::new(reader).lines().flatten() {
            if line.starts_with('#') {
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
                        .and_then(|dt| dt.try_into().ok());
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

fn get_current_state_info(server: &str) -> anyhow::Result<StateInfo> {
    let res = ureq::get(&latest_state_url(server)).call()?;

    Ok(StateInfo::try_from_reader(res.into_reader())?)
}

fn get_state_info(server: &str, seqno: u64) -> anyhow::Result<StateInfo> {
    // TODO
    eprintln!("getting state for seqno {}", seqno);
    let res = ureq::get(&state_url(server, seqno)).call()?;

    Ok(StateInfo::try_from_reader(res.into_reader())?)
}

fn latest_state_url(server: &str) -> String {
    format!("{}/state.txt", server)
}

fn state_url(server: &str, seqno: u64) -> String {
    let triplet = seqno_to_triplet(seqno);
    format!(
        "{}/{:03}/{:03}/{:03}.state.txt",
        server, triplet.0, triplet.1, triplet.2
    )
}

fn osc_url(server: &str, seqno: u64) -> String {
    let triplet = seqno_to_triplet(seqno);
    format!(
        "{}/{:03}/{:03}/{:03}.osc.gz",
        server, triplet.0, triplet.1, triplet.2
    )
}

fn seqno_to_triplet(seqno: u64) -> (u16, u16, u16) {
    let hi = (seqno / 1_000_000) as u16;
    let md = ((seqno % 1_000_000) / 1000) as u16;
    let lo = (seqno % 1000) as u16;

    (hi, md, lo)
}
