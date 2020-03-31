use clap::{App, Arg};
use std::io;
use std::fs;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub listen: String,
    pub timeout: u32,
    pub servers: Vec<String>,
}

pub fn new_upstream(path: &str) -> io::Result<Config> {
    let contents = fs::read_to_string(path)?;
    let conf = serde_json::from_str(&contents);
    match conf {
        Ok(cf) => Ok(cf),
        Err(_) => Err(io::Error::new(io::ErrorKind::Other, "serde jsom failed")),
    }
}

pub struct Args<'a> {
    pub cmd: clap::ArgMatches<'a>,
}

pub fn new_args<'a>() -> Args<'a> {
    let matchs = App::new("iperfp")
        .version("1.0")
        .author("iake123@163.com")
        .about("iperf proxy")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .takes_value(true)
                .value_name("PATH")
                .help("Set a custom config file")
                .required(true),
        )
        .get_matches();
    return Args { cmd: matchs };
}

impl<'a> Args<'a> {
    pub fn get_path(&self) -> Option<&str> {
        return self.cmd.value_of("config");
    }
}