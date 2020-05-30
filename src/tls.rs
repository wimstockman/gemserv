extern crate openssl;
extern crate tokio_openssl;
use std::collections::HashMap;

use openssl::error::ErrorStack;
use openssl::ssl::NameType;
use openssl::ssl::SniError;
use openssl::ssl::SslContextBuilder;
use openssl::ssl::SslVersion;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use crate::config;

pub fn acceptor_conf(cfg: config::Config) -> Result<SslAcceptor, ErrorStack> {
    let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
    acceptor.set_min_proto_version(Some(SslVersion::TLS1_2))?;
    let mut map = HashMap::new();
    let mut num = 1;
    for server in cfg.server.iter() {
        let mut ctx = SslContextBuilder::new(SslMethod::tls())?;
        match ctx.set_private_key_file(&server.key, SslFiletype::PEM) {
            Ok(c) => c,
            Err(e) => {
                log::error!("Error: Can't load key file");
                return Err(e);
            }
        };
        match ctx.set_certificate_chain_file(&server.cert) {
            Ok(c) => c,
            Err(e) => {
                log::error!("Error: Can't load cert file");
                return Err(e);
            }
        };
        let ctx = ctx.build();
        map.insert(server.hostname.clone(), ctx.clone());
        if num == 1 {
            map.insert("default".to_string(), ctx);
            num += 1;
        }
    }

    let ctx_builder = &mut *acceptor;
    ctx_builder.set_servername_callback(move |ssl, _alert| -> Result<(), SniError> {
        ssl.set_ssl_context({
            let hostname = ssl.servername(NameType::HOST_NAME);
            if let Some(host) = hostname {
                if let Some(ctx) = map.get(host) {
                    &ctx
                } else {
                    &map.get(&"default".to_string()).expect("Can't get default")
                }
            } else {
                &map.get(&"default".to_string()).expect("Can't get default")
            }
        })
        .expect("Can't get sni");
        Ok(())
    });
    Ok(acceptor.build())
}
