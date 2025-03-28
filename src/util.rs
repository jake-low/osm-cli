use std::fmt;

use clap::ValueEnum;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Format {
    Xml,
    Json,
}

impl Format {
    pub fn mimetype(&self) -> &str {
        match self {
            Format::Xml => "application/xml",
            Format::Json => "application/json",
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Format::Xml => "xml",
            Format::Json => "json",
        };
        write!(f, "{}", s)
    }
}

/// Returns the mime-type portion of the Content-Type header, if present
pub fn content_type<T>(res: &ureq::http::Response<T>) -> Option<&str> {
    res.headers()
        .get("Content-Type")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(";").next())
}
