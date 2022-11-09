use std::{
    fs,
    net::{IpAddr, Ipv4Addr},
    process::exit,
    sync::Arc,
    time::Duration,
};

use rustls::{Certificate, PrivateKey};
use stubborn_io::{ReconnectOptions, StubbornTcpStream};
use tarpc::{
    serde_transport::Transport,
    server::{BaseChannel, Channel},
    tokio_serde::formats::Json,
};
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio_rustls::{TlsAcceptor, TlsConnector};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::{
    rpc::{Authd, AuthdSession},
    types::Config,
};

pub mod rpc;
mod socketname;
pub mod types;

pub use socketname::SocketName;
pub use types::Shell;

/// Hosts an authd server
pub async fn listen_server() -> anyhow::Result<()> {
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

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

/// Connect to authd over TLS, already knowing + trusting its certificate (if we don't get MITM).
///
/// The server_name is used for SNI. Setting it to localhost is fine for testing.
pub async fn connect_client<A: ToSocketAddrs + Unpin + Clone + Send + Sync + 'static>(
    addr: A,
    cert: &rustls::Certificate,
    server_name: &str,
) -> anyhow::Result<rpc::AuthdClient> {
    let reconnect_opts = ReconnectOptions::new()
        .with_exit_if_first_connect_fails(true)
        .with_retries_generator(|| std::iter::repeat(Duration::from_secs(1)));
    let tcp_stream = StubbornTcpStream::connect_with_options(addr, reconnect_opts).await?;

    let mut roots = rustls::RootCertStore::empty();
    roots.add(cert).unwrap();

    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));
    let servername = rustls::ServerName::try_from(server_name).unwrap();
    let transport = Transport::from((
        connector.connect(servername, tcp_stream).await?,
        tarpc::tokio_serde::formats::Json::default(),
    ));

    Ok(rpc::AuthdClient::new(tarpc::client::Config::default(), transport).spawn())
}
