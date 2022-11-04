use std::{fs, sync::Arc, process::exit, net::{IpAddr, Ipv6Addr}, future, time::Duration};

use futures::StreamExt;
use stubborn_io::{ReconnectOptions, StubbornTcpStream};
use tarpc::{tokio_serde::formats::Json, server::Channel, serde_transport::Transport};
use tokio::net::ToSocketAddrs;
use tokio_rustls::TlsConnector;

use crate::{types::Config, rpc::{AuthdSession, Authd}};

pub mod types;
pub mod rpc;
mod socketname;

pub use socketname::SocketName;

/// Hosts an authd server
pub async fn listen_server() -> anyhow::Result<()> {
    let contents = fs::read_to_string("example.toml")?;
    let config = Arc::new(toml::from_str::<Config>(&contents)?);

    if !config.check_dup() {
        exit(1);
    }

    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), 13699);

    // JSON transport is provided by the json_transport tarpc module. It makes it easy
    // to start up a serde-powered json serialization strategy over TCP.
    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;
    tracing::info!("Listening on port {}", listener.local_addr().port());
    listener.config_mut().max_frame_length(usize::MAX);
    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(tarpc::server::BaseChannel::with_defaults)
        .map(|channel| {
            let server = AuthdSession::new(config.clone());
            channel.execute(server.serve())
        })
        .for_each(|_| async {})
        .await;

    Ok(())
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
        .with_exit_if_first_connect_fails(false)
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
