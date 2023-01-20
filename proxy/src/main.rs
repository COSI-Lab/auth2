mod rpc;

use libcosiauthd::{Authd, AuthdClient, Shell, SocketName};
use std::{fs, path::Path, sync::Arc};
use stubborn_io::StubbornTcpStream;
use tarpc::{
    serde_transport::Transport,
    server::{BaseChannel, Channel},
    tokio_serde::formats::Json,
};
use tokio::net::{ToSocketAddrs, UnixListener};
use tokio_rustls::TlsConnector;
use tracing::{info, warn};
use tracing_subscriber::FmtSubscriber;

use crate::rpc::ProxySession;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ProxyConfig {
    host: SocketName,
    cert: String,
    shells_root: String,
    shells: Vec<Shell>,
    home_root: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let contents = fs::read_to_string("/etc/auth/authd.toml").expect("read config");
    let config = toml::from_str::<ProxyConfig>(&contents).expect("parse config");

    Ok(())
}

/// Connect to authd over TLS, already knowing + trusting its certificate (if we don't get MITM).
///
/// The server_name is used for SNI. Setting it to localhost is fine for testing.
async fn connect_client<A: ToSocketAddrs + Unpin + Clone + Send + Sync + 'static>(
    addr: A,
    cert: &rustls::Certificate,
    server_name: &str,
) -> anyhow::Result<AuthdClient> {
    let tcp_stream = StubbornTcpStream::connect(addr)
        .await
        .expect("failed to create StubbornTcpStream connection");

    let mut roots = rustls::RootCertStore::empty();
    roots.add(cert).unwrap();

    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));
    let servername = rustls::ServerName::try_from(server_name)?;
    let transport = Transport::from((
        connector.connect(servername, tcp_stream).await?,
        tarpc::tokio_serde::formats::Json::default(),
    ));

    Ok(AuthdClient::new(tarpc::client::Config::default(), transport).spawn())
}

/// Serve a TARPC server waiting for a unix socket
async fn listen_unix<P>(socket: P, config: ProxyConfig) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    // Connect to authd
    let cert = rustls::Certificate(config.cert.into_bytes());
    let addr = &config.host.to_socket_addr();

    let client = connect_client(config.host, &cert, "localhost").await?;

    let listener = UnixListener::bind(socket)?;

    info!("Listening to {:?}", listener.local_addr().unwrap());

    loop {
        let session = session.clone();

        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("New connection from {:?}", addr);

                let tport = tarpc::serde_transport::Transport::from((stream, Json::default()));
                let channel = BaseChannel::with_defaults(tport);

                channel.execute(session.serve()).await;
            }
            Err(err) => {
                warn!("Connection failed {:?}", err)
            }
        }
    }
}
