mod app;
mod config;
mod error;
mod models;
mod routes;
mod services;

use config::AppConfig;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = AppConfig::from_env();
    let address = config.socket_addr()?;
    let app = app::build(config).await?;
    let listener = TcpListener::bind(address).await?;

    tracing::info!("web-file-browser 已启动，监听地址: http://{address}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}
