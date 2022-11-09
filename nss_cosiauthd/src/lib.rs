use authd::{Shell, SocketName};
use std::io;
use std::sync::Mutex;
use tokio::runtime::Runtime;

use crate::client::ClientAccessControl;
use crate::group::AuthdGroup;
use crate::passwd::AuthdPasswd;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

extern crate libc;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate libnss;

mod client;
mod group;
mod passwd;

#[derive(serde::Deserialize)]
struct NssConfig {
    host: SocketName,
    cert: String,
    shells_root: String,
    shells: Vec<Shell>,
    home_root: String,
}

/// Prints to error out only if `debug_assertions` are on
#[macro_export]
macro_rules! debug {
    ($($e:expr),+) => {
        {
            #[cfg(debug_assertions)]
            {
                use std::fs::OpenOptions;
                use std::io::Write;

                let s = format!($($e),+);
                let mut file = OpenOptions::new().append(true).open("/tmp/nss_cosiauthd").unwrap();
                writeln!(file, "{}", &s).unwrap();
                eprintln!("{}", s)
            }
        }
    }
}

lazy_static! {
    static ref RPC: Mutex<ClientAccessControl> = {
        // a builder for `FmtSubscriber`.
        let subscriber = FmtSubscriber::builder()
            // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
            // will be written to stdout.
            .with_max_level(Level::TRACE)
            // completes the builder.
            .finish();

        tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

        Mutex::new(ClientAccessControl::default())
    };

    static ref RT: io::Result<Runtime> = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_io()
        .enable_time()
        .build();

    static ref CFG: anyhow::Result<NssConfig> = {
        let contents = std::fs::read("/etc/auth/nss_cosiauthd.toml")?;
        let toml = toml::from_slice(&contents)?;
        Ok(toml)
    };
}

libnss_passwd_hooks!(cosiauthd, AuthdPasswd);
libnss_group_hooks!(cosiauthd, AuthdGroup);
