mod config;
mod rpc;

use crate::{config::Config, rpc::AuthdSession};

use libcosiauthd::Authd;
use rustls::{Certificate, PrivateKey};
use std::{
    fs,
    net::{IpAddr, Ipv4Addr},
    process::exit,
    sync::Arc,
};
use tarpc::{
    server::{BaseChannel, Channel},
    tokio_serde::formats::Json,
};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    listen_server().await
}

/// Hosts an authd server
async fn listen_server() -> anyhow::Result<()> {
    let contents = fs::read_to_string("/etc/auth/authd.toml").expect("read config");
    let config = Arc::new(toml::from_str::<Config>(&contents).expect("parse config"));

    if !config.check_dup() {
        exit(1);
    }

    let server_addr = (IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8765);

    let tls_config = Arc::new(
        rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(
                vec![Certificate(std::fs::read(&config.cert).expect("read cert"))],
                PrivateKey(std::fs::read(&config.key).expect("read key")),
            )?,
    );

    let tls_config = tls_config.clone();
    let acceptor: TlsAcceptor = tls_config.into();

    let listener = TcpListener::bind(&server_addr).await.expect("tcp bind");

    tracing::info!("listening on {:?}", server_addr);

    loop {
        let acceptor = acceptor.clone();
        let (stream, peer_addr) = listener.accept().await.expect("tcp accept");

        let cloned = config.clone();

        tokio::spawn(async move {
            let stream = acceptor.accept(stream).await.expect("accepting tls");
            let tport = tarpc::serde_transport::Transport::from((stream, Json::default()));
            let channel = BaseChannel::with_defaults(tport);
            let session = AuthdSession::new(cloned);

            tracing::info!("new connection: {:?}", peer_addr);
            channel.execute(session.serve()).await;
        });
    }
}
