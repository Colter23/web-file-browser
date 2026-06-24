mod app;
mod config;
mod error;
mod models;
mod routes;
mod services;

use config::AppConfig;
use std::{
    env,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::args().any(|arg| arg == "--healthcheck") {
        run_healthcheck()?;
        return Ok(());
    }

    tracing_subscriber::fmt::init();

    let config = AppConfig::load()?;
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

fn run_healthcheck() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load()?;
    let address = SocketAddr::from(([127, 0, 0, 1], config.port));
    let timeout = Duration::from_secs(2);
    let mut stream = TcpStream::connect_timeout(&address, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;

    stream.write_all(b"GET /api/ready HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")?;

    let mut buffer = [0_u8; 128];
    let bytes = stream.read(&mut buffer)?;
    let response = std::str::from_utf8(&buffer[..bytes])?;
    let status_line = response.lines().next().unwrap_or_default();
    if status_line.contains(" 200 ") {
        Ok(())
    } else {
        Err(format!("健康检查未就绪: {status_line}").into())
    }
}
