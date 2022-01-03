extern crate serde_derive;
extern crate serde_regex;
extern crate toml;
use crate::lib::errors;
use std::collections::HashMap;
use std::env;
use std::net;
use std::net::ToSocketAddrs;
use std::path;
use tokio::fs;
use tokio::io;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Deserialize, Clone)]
pub struct Location {
    #[serde(with = "serde_regex")]
    pub location: regex::Regex,
    pub lang: Option<String>,
    pub meta: Option<String>,
    pub charset: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub port: Option<u16>,
    pub host: Option<String>,
    pub interface: Option<Vec<net::SocketAddr>>,
    pub log: Option<String>,
    pub server: Vec<Server>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Server {
    pub hostname: String,
    pub dir: String,
    pub key: String,
    pub cert: String,
    pub index: Option<String>,
    pub lang: Option<String>,
    pub locations: Option<Vec<Location>>,
    #[cfg(feature = "cgi")]
    pub cgi: Option<bool>,
    #[cfg(feature = "cgi")]
    pub cgipath: Option<String>,
    #[cfg(any(feature = "cgi", feature = "scgi"))]
    pub cgienv: Option<HashMap<String, String>>,
    pub usrdir: Option<bool>,
    #[cfg(feature = "proxy")]
    pub proxy: Option<HashMap<String, String>>,
    #[cfg(feature = "proxy")]
    pub proxy_all: Option<String>,
    pub redirect: Option<HashMap<String, String>>,
    #[cfg(feature = "scgi")]
    pub scgi: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct ServerCfg {
    //    pub port: u16,
    pub server: Server,
}

impl Config {
    pub async fn new() -> Result<Config> {
        let args: Vec<String> = env::args().collect();
        let mut p = path::PathBuf::new();
        if args.len() != 2 {
            p.push("/usr/local/etc/gemserv.conf");
            if !p.exists() {
                return Err(Box::new(errors::GemError(
                    "Please run with the path to the config file. \
                    Or create the config as /usr/local/etc/gemserv.conf"
                        .to_string(),
                )));
            }
        } else {
            p.push(&args[1]);
        }

        let fd = fs::read_to_string(p).await.unwrap();
        let mut config: Config = match toml::from_str(&fd) {
            Ok(c) => c,
            Err(e) => return Err(Box::new(e)),
        };

        // We do this to convert u labels to a labels
        for mut srv in &mut config.server {
            let alabel = idna::domain_to_ascii(&srv.hostname)?;
            srv.hostname = alabel;
        }

        if config.host.is_some() || config.port.is_some() {
            eprintln!(
                "The host/port keys are depricated in favor \
            of interface and may be removed in the future."
            );
        }

        if config.interface.is_some() && (config.host.is_some() || config.port.is_some()) {
            return Err(Box::new(errors::GemError(
                "You need to specify either host/port or interface".into(),
            )));
        } else if config.interface.is_none() && config.host.is_none() && config.port.is_none() {
            return Err(Box::new(errors::GemError(
                "You need to specify either host/port or interface".into(),
            )));
        } else if config.host.is_some() && config.port.is_some() {
            let mut addr: Vec<std::net::SocketAddr> = Vec::new();
            addr.push(
                format!(
                    "{}:{}",
                    &config.host.to_owned().unwrap(),
                    &config.port.unwrap()
                )
                .to_socket_addrs()?
                .next()
                .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?,
            );
            config.interface = Some(addr);
            return Ok(config);
        } else if let Some(ref mut i) = config.interface {
            i.sort_by(|a, b| a.port().cmp(&b.port()));
            i.dedup();
            return Ok(config);
        }
        Err(Box::new(errors::GemError(
            "You need to specify either host/port or interface".into(),
        )))
    }
    pub fn to_map(&self) -> HashMap<String, ServerCfg> {
        let mut map = HashMap::new();
        for srv in &self.server {
            map.insert(
                srv.hostname.clone(),
                ServerCfg {
                    //    port: self.port.clone(),
                    server: srv.clone(),
                },
            );
        }
        map
    }
}
