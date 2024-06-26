extern crate serde_derive;
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
    #[cfg(feature = "authlocation")]
    pub authlocation: Vec<AuthLocation>,
    #[cfg(feature = "cgi")]
    pub cgi: Option<bool>,
    #[cfg(feature = "cgi")]
    pub cgipath: Option<Vec<String>>,
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

#[cfg(feature = "authlocation")]
#[derive(Debug, Deserialize, Clone)]
pub struct AuthLocation {
    pub auth_basic: String,
    pub index: Option<String>,
    pub path: String,
    pub root: String,
    #[serde(skip)]
    pub hashkeys: Vec<String>,
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

        #[cfg(feature = "authlocation")]
        for serv in &mut config.server {
            let authlo = &mut serv.authlocation;
            for authloc in authlo.iter_mut() {
                let mut kfile = path::PathBuf::new();
                kfile.push(&authloc.auth_basic);
                if !kfile.exists() {
                    return Err(Box::new(errors::GemError(
                        "Hash table file doesn't exist".to_string(),
                    )));
                } else {
                    let toml_str = fs::read_to_string(kfile).await.unwrap();
                    let table: toml::Value = toml::from_str(&toml_str)?;
                    let hashkeys = table["hashkeys"].as_array().ok_or("Hahkeys not found")?;
                    let hashkeys: Vec<String> = hashkeys
                        .iter()
                        .map(|v| v.as_str().unwrap().to_string())
                        .collect();
                    authloc.hashkeys = hashkeys.clone();
                }
            }
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
