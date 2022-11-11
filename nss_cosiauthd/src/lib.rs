use authd::{Shell, SocketName};
use tracing::info;
use std::io;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use tracing_subscriber::fmt::writer::MakeWriterExt;

use crate::client::ClientAccessControl;
use crate::group::AuthdGroup;
use crate::passwd::AuthdPasswd;

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

lazy_static! {
    static ref RPC: Mutex<ClientAccessControl> = {
        // Log all events to a rolling log file.
        let logfile = tracing_appender::rolling::hourly("/tmp", "nss_cosiauthd.log");

        // Log `INFO` and above to stdout.
        let stdout = std::io::stdout.with_max_level(tracing::Level::TRACE);

        tracing_subscriber::fmt()
            // Combine the stdout and log file `MakeWriter`s into one
            // `MakeWriter` that writes to both
            .with_writer(stdout.and(logfile))
            .init();

        info!("logging ready");

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
