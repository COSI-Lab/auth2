use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use libnss::interop::Response;
use tokio::time::sleep_until;
use tracing::{error, info, warn};
use trust_dns_resolver::TokioAsyncResolver;

use crate::{CFG, RT};

#[derive(Default)]
pub struct ClientAccessControl {
    client: Arc<Mutex<Option<AuthdClient>>>,
    latest_ts: Arc<Mutex<Option<Instant>>>,
}

impl ClientAccessControl {
    pub fn with_client<O>(
        &mut self,
        f: impl FnOnce(&mut AuthdClient) -> Response<O>,
    ) -> Response<O> {
        let Ok(rt) = &*RT else {
            error!("Runtime unavialable");
            return Response::Unavail;
        };
        let Ok(cfg) = &*CFG else {
            error!("Configuration unavialable");
            return Response::Unavail;
        };

        let _guard = rt.enter();
        let mut lts = self.latest_ts.lock().unwrap();
        *lts = Some(std::time::Instant::now() + Duration::from_secs(30));
        let cl = self.client.clone();
        let latest_ts = self.latest_ts.clone();
        tokio::spawn(async move {
            loop {
                let dur = latest_ts.lock().unwrap().unwrap_or(Instant::now()).into();
                sleep_until(dur).await;
                // make sure it wasn't moved forward while we were sleeping
                if latest_ts.lock().unwrap().unwrap_or(Instant::now()) < Instant::now() {
                    *cl.lock().unwrap() = None;
                    warn!(
                        "nss_cosiauthd: ClientAccessControl: client timed out, closing connection."
                    );
                    break;
                }
            }
        });

        let Ok(resolver) = TokioAsyncResolver::tokio_from_system_conf() else {
            return Response::Unavail
        };

        let final_sockaddr = match &cfg.host {
            SocketName::Dns(name, port) => {
                let Ok(ips) = rt.block_on(resolver.lookup_ip(name)) else {
                    warn!("Failed to do DNS lookup");
                    return Response::Unavail;
                };

                let Some(ip) = ips.iter().next() else {
                    warn!("No ips were returned");
                    return Response::Unavail;
                };

                std::net::SocketAddr::new(ip, *port)
            }
            SocketName::Addr(sa) => *sa,
        };

        info!(
            "nss_cosiauthd: ClientAccessControl: connecting to {}",
            final_sockaddr
        );

        let mut client = self.client.lock().unwrap();
        if client.is_none() {
            let Ok(cert) = std::fs::read(&cfg.cert) else {
                error!("Failed to read cert");
                return Response::Unavail;
            };

            let c = rt.block_on(authd::connect_client(
                final_sockaddr,
                &rustls::Certificate(cert),
                "localhost",
            ));

            match c {
                Ok(c) => *client = Some(c),
                Err(_) => return Response::Unavail,
            }
        }

        info!("client ready");

        // SAFETY: `unwrap()` will never panic here because if client was `None` it would have been
        // overwritten to `Some(c)` in the previous block or we returned Response::Unavail
        f(client.as_mut().unwrap())
    }
}
