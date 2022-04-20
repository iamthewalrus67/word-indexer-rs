use std::{error::Error, fmt};

use configparser::ini::Ini;

#[derive(Debug)]
pub struct Config {
    pub indexing_threads: u64,
    pub indir: String,
    pub out_by_a: String,
    pub out_by_n: String,
}

impl Config {
    pub fn new(indexing_threads: u64, indir: &str, out_by_a: &str, out_by_n: &str) -> Self {
        Self {
            indexing_threads: indexing_threads,
            indir: indir.replace(&['\'', '\"'][..], ""),
            out_by_a: out_by_a.replace(&['\'', '\"'][..], ""),
            out_by_n: out_by_n.replace(&['\'', '\"'][..], ""),
        }
    }
}

#[derive(Debug)]
pub struct ConfigParseError {}

impl Error for ConfigParseError {}

impl fmt::Display for ConfigParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Config parsing error")
    }
}

pub fn parse_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let mut ini = Ini::new();
    let config_file = std::fs::read_to_string(path)?;
    ini.read(config_file)?;

    let indexing_threads = match ini.getuint("default", "indexing_threads") {
        Ok(indexing_threads) => match indexing_threads {
            Some(v) => v,
            None => return Err(Box::new(ConfigParseError {})),
        },
        _ => return Err(Box::new(ConfigParseError {})),
    };

    let indir = match ini.get("default", "indir") {
        Some(v) => v,
        None => return Err(Box::new(ConfigParseError {})),
    };

    let out_by_a = match ini.get("default", "out_by_a") {
        Some(v) => v,
        None => return Err(Box::new(ConfigParseError {})),
    };

    let out_by_n = match ini.get("default", "out_by_n") {
        Some(v) => v,
        None => return Err(Box::new(ConfigParseError {})),
    };

    let config = Config::new(indexing_threads, &indir, &out_by_a, &out_by_n);

    return Ok(config);
}
