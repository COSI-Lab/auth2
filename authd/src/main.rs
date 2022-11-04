use authd::listen_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    listen_server().await
}
