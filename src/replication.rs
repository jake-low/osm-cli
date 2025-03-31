use std::io::{self, BufRead, Read, Write};
use std::thread::sleep;
use std::time::Duration;

use chrono::{DateTime, Utc};
use clap::Parser;
use log::{debug, info};
use serde::{Deserialize, Serialize};

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

    /// Only print URLs of replication files (not their seqnos and timestamps). This
    /// is faster since it doesn't require a GET request per line of output.
    #[arg(long)]
    urls_only: bool,

    /// Server short name [minute, changesets] or URL
    server: String,
}

pub fn run(args: &CliArgs) -> anyhow::Result<()> {
    env_logger::init();

    let server = match &args.server[..] {
        "minute" => ReplicationServer {
            base_url: "https://planet.openstreetmap.org/replication/minute".to_string(),
            current_state_path: "state.txt".to_string(),
            state_file_format: StateFileFormat::Text,
            state_file_extension: ".state.txt".to_string(),
            data_file_extension: ".osc.gz".to_string(),
        },
        "changesets" => ReplicationServer {
            base_url: "https://planet.openstreetmap.org/replication/changesets".to_string(),
            current_state_path: "state.yaml".to_string(),
            state_file_format: StateFileFormat::Yaml,
            state_file_extension: ".state.txt".to_string(),
            data_file_extension: ".osm.gz".to_string(),
        },
        url => ReplicationServer {
            base_url: url.to_string(),
            // TODO: these values should probably be user-configurable
            current_state_path: "state.txt".to_string(),
            state_file_format: StateFileFormat::Text,
            state_file_extension: ".state.txt".to_string(),
            data_file_extension: ".osc.gz".to_string(),
        },
    };

    let start_seqno = if let Some(since) = &args.since {
        let since: DateTime<Utc> = DateTime::parse_from_rfc2822(since)
            .ok()
            .or_else(|| DateTime::parse_from_rfc3339(since).ok())
            .expect("failed to parse --since as timestamp")
            .into();
        info!("binary searching to find starting sequence number");

        server.timestamp_to_seqno(since)?
    } else if let Some(seqno) = args.seqno {
        seqno
    } else {
        panic!("require either --since or --seqno");
    };

    let mut seqno = start_seqno;
    let mut latest = server.get_current_state_info()?;

    loop {
        if seqno >= latest.seqno {
            if args.watch {
                sleep(Duration::from_secs(60));
                latest = server.get_current_state_info()?;
            } else {
                break;
            }
        }

        seqno += 1;

        let url = server.data_url(seqno);
        if args.urls_only {
            writeln!(io::stdout(), "{}", url)?;
        } else {
            let info = server.get_state_info(seqno)?;
            writeln!(
                io::stdout(),
                "{} {} {}",
                info.seqno,
                info.timestamp
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                url
            )?;
        }
    }

    Ok(())
}

struct ReplicationServer {
    base_url: String,
    current_state_path: String,
    state_file_format: StateFileFormat,
    state_file_extension: String,
    data_file_extension: String,
}

#[derive(Debug, PartialEq, Eq)]
enum StateFileFormat {
    Yaml,
    Text,
}

impl ReplicationServer {
    fn timestamp_to_seqno(&self, timestamp: DateTime<Utc>) -> anyhow::Result<u64> {
        let mut upper = self.get_current_state_info()?;

        if upper.timestamp < timestamp || upper.seqno == 0 {
            return Ok(upper.seqno);
        }

        let mut guess: u64 = 0;
        let mut lower: Option<StateInfo>;

        // find a state file that is below the required timestamp
        loop {
            lower = self.get_state_info(guess).ok();

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
            let step = f64::ceil((desired_time_step as f64) * seqno_rate);
            guess = lower.seqno + f64::max(step, 1.0) as u64;
            if guess == upper.seqno {
                guess -= 1;
            }

            // dbg!(&guess);
            debug!("guessing {}", &guess);

            let split = self.get_state_info(guess);

            // TODO: what if split.is_none() (i.e. guess not found)?
            // we should walk up+down to find a nearby split candidate
            let split = split.unwrap();

            // dbg!(&split);

            if split.timestamp < timestamp {
                debug!("guess was too low");
                lower = split;
            } else {
                debug!("guess was too high");
                upper = split;
            }

            if lower.seqno + 1 >= upper.seqno {
                return Ok(lower.seqno);
            }
            debug!("trying again");
        }
    }

    fn get_current_state_info(&self) -> anyhow::Result<StateInfo> {
        let url = self.latest_state_url();
        info!("GET {}", &url);
        let res = ureq::get(&url).call()?;
        let mut body = res.into_body();

        let state_info: StateInfo = match self.state_file_format {
            StateFileFormat::Yaml => serde_yaml::from_reader(body.as_reader())?,
            StateFileFormat::Text => StateInfo::try_from_reader(body.as_reader())?,
        };

        Ok(state_info)
    }

    fn get_state_info(&self, seqno: u64) -> anyhow::Result<StateInfo> {
        let url = self.state_url(seqno);
        info!("GET {}", &url);
        let res = ureq::get(&url).call()?;
        let mut body = res.into_body();

        let state_info: StateInfo = match self.state_file_format {
            StateFileFormat::Yaml => serde_yaml::from_reader(body.as_reader())?,
            StateFileFormat::Text => StateInfo::try_from_reader(body.as_reader())?,
        };

        Ok(state_info)
    }

    fn latest_state_url(&self) -> String {
        format!("{}/{}", self.base_url, self.current_state_path)
    }

    fn state_url(&self, seqno: u64) -> String {
        let seqno = self.seqno_for_url(seqno);
        let triplet = seqno_to_triplet(seqno);
        format!(
            "{}/{:03}/{:03}/{:03}{}",
            self.base_url, triplet.0, triplet.1, triplet.2, self.state_file_extension
        )
    }

    fn data_url(&self, seqno: u64) -> String {
        let seqno = self.seqno_for_url(seqno);
        let triplet = seqno_to_triplet(seqno);
        format!(
            "{}/{:03}/{:03}/{:03}{}",
            self.base_url, triplet.0, triplet.1, triplet.2, self.data_file_extension
        )
    }

    fn seqno_for_url(&self, seqno: u64) -> u64 {
        if self.base_url == "https://planet.openstreetmap.org/replication/changesets" {
            // HACK: the changeset replication sequence numbers are off by one from the filenames,
            // so we need to increment the given seqno when constructing a URL.
            //
            // For example: if /replication/changesets/state.yaml says the most recent sequence
            // number is 1234567, then /replication/changesets/001/234/568.state.txt (not
            // 567.state.txt!) will be identical in content to state.yaml, and fetching
            // /replication/changesets/001/234/568.osm.gz (not 567.osm.gz!) will return
            // changesets from the roughly one-minute window prior to the last_run time
            // listed in state.yaml.
            //
            // Since the sequence numbers in state.yaml and the individual NNN.state.txt files
            // agree with each other, this program treats those as the correct values, and uses
            // those values when comparing to `--seqno` and when printing sequence numbers in
            // the output. If you need sequence numbers you are advised to treat the seqnos in
            // the first column of this program's output as correct, and treat the URLs as opaque
            // (do not parse the sequence numbers out of the URLs, because they are off by one).
            //
            // See also https://osmus.slack.com/archives/C1VE7JM9T/p1732434693862139?thread_ts=1727214687.839279&cid=C1VE7JM9T
            seqno.saturating_add(1)
        } else {
            // For other replication endpoints the sequence numbers in the state files agree with
            // those in the URLs, so this function is just the identity function
            seqno
        }
    }
}

fn seqno_to_triplet(seqno: u64) -> (u16, u16, u16) {
    let hi = (seqno / 1_000_000) as u16;
    let md = ((seqno % 1_000_000) / 1000) as u16;
    let lo = (seqno % 1000) as u16;

    (hi, md, lo)
}

#[derive(Debug, Serialize, Deserialize)]
struct StateInfo {
    #[serde(alias = "sequence")]
    seqno: u64,

    #[serde(alias = "last_run")]
    timestamp: DateTime<Utc>,
}

impl StateInfo {
    fn try_from_reader(reader: impl Read) -> anyhow::Result<Self> {
        let mut seqno: Option<u64> = None;
        let mut timestamp: Option<DateTime<Utc>> = None;

        for line in io::BufReader::new(reader).lines().map_while(Result::ok) {
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
                        .map(|dt| dt.into());
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
