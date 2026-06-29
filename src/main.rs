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
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream},
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
    let address = healthcheck_address(&config)?;
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

fn healthcheck_address(config: &AppConfig) -> Result<SocketAddr, std::net::AddrParseError> {
    let address = config.socket_addr()?;
    let ip = match address.ip() {
        IpAddr::V4(ip) if ip.is_unspecified() => IpAddr::V4(Ipv4Addr::LOCALHOST),
        IpAddr::V6(ip) if ip.is_unspecified() => IpAddr::V6(Ipv6Addr::LOCALHOST),
        ip => ip,
    };
    Ok(SocketAddr::new(ip, address.port()))
}

#[cfg(test)]
mod tests {
    use super::healthcheck_address;
    use crate::config::AppConfig;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn healthcheck_uses_loopback_for_unspecified_ipv4_bind() {
        let mut config = AppConfig::load_from_file("missing-config.json".into()).unwrap();
        config.bind_address = "0.0.0.0".to_string();
        config.port = 18080;

        let address = healthcheck_address(&config).unwrap();

        assert_eq!(address.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(address.port(), 18080);
    }

    #[test]
    fn healthcheck_uses_loopback_for_unspecified_ipv6_bind() {
        let mut config = AppConfig::load_from_file("missing-config.json".into()).unwrap();
        config.bind_address = "::".to_string();
        config.port = 18080;

        let address = healthcheck_address(&config).unwrap();

        assert_eq!(address.ip(), IpAddr::V6(Ipv6Addr::LOCALHOST));
        assert_eq!(address.port(), 18080);
    }

    #[test]
    fn healthcheck_keeps_specific_bind_address() {
        let mut config = AppConfig::load_from_file("missing-config.json".into()).unwrap();
        config.bind_address = "192.168.1.10".to_string();
        config.port = 18080;

        let address = healthcheck_address(&config).unwrap();

        assert_eq!(address.ip(), IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)));
        assert_eq!(address.port(), 18080);
    }
}
