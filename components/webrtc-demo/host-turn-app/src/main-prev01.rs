
fn main() {
    println!("Hello, world!");
    main_demo1();
}

#[tokio::main]
async fn main_test1() {
    let local_address = netdev::net::ip::get_local_ipaddr().expect("Could not determine a local address");
    dbg!(local_address);
    let gateway_address = find_gateway_address(local_address).expect("Could not determine a gateway address");
    dbg!(gateway_address);


    // we have two ways of obtaining the external address (at this point; we could also try to use STUN)
    let natpmp = crab_nat::natpmp::external_address(gateway_address, None);
    let 
}

async fn try_natpmp(local_address: IpAddr) -> Ipv4Addr {
    let gateway_address = find_gateway_address(local_address).expect("Could not determine a gateway address");
    dbg!(gateway_address);
    let external_address = crab_nat::natpmp::external_address(gateway_address, None).await;
    dbg!(external_address);
    external_address
}

async fn try_igd()

#[tokio::main]
async fn main_demo1() {
    println!("calling [main_demo1]");
    let (mapping, bind, external) = setup_crab().await;
    setup_turn(bind, external).await;
}


use std::sync::Arc;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::num::NonZeroU16;
use std::time::Duration;

use turn_server::{ config::{Config, Turn, Interface, Transport, Api, Log, LogLevel, Auth},  };

const BIND_PORT: NonZeroU16 = NonZeroU16::new(9701).unwrap();

// see:
// - https://github.com/mycrl/turn-rs/blob/v3.4.0/src/main.rs
// - https://github.com/mycrl/turn-rs/blob/v3.4.0/turn-server.toml
async fn setup_turn(bind: SocketAddr, external: SocketAddr) {
    // we will manually build the config instead of trying to parse a config file or CLI args
    let config = Config {
        turn: Turn {
            realm: "localhost".to_string(),
            interfaces: vec![Interface {
                transport: Transport::UDP,
                bind,
                external
            }]
        },
        api: Api { bind: "127.0.0.1:3000".parse().unwrap() },
        log: Log { level: LogLevel::Info },
        auth: Auth {
            static_credentials: Default::default(),
            static_auth_secret: Some("some_password_123".to_string())
        }
    };

    // TODO we should setup simple_logger?
    // simple_logger::init_with_level(config.log.level.as_level())?;

    // config
    // call in "tokio::main":
    turn_server::startup(config.into()).await.unwrap();
}



use crab_nat::{InternetProtocol, PortMapping, PortMappingOptions};

// see: https://github.com/ryco117/crab_nat/blob/main/examples/client.rs
async fn setup_crab() -> (PortMapping, SocketAddr, SocketAddr) {
    let local_address = netdev::net::ip::get_local_ipaddr().expect("Could not determine a local address");
    dbg!(local_address);
    let gateway_address = find_gateway_address(local_address).expect("Could not determine a gateway address");
    dbg!(gateway_address);
    let timeout = crab_nat::TimeoutConfig {
        initial_timeout: Duration::new(1, 0),
        max_retries: 10,
        max_retry_timeout: Some(Duration::new(10, 0)),
    };
    let external_address = crab_nat::natpmp::external_address(gateway_address, Some(timeout)).await
        // .unwrap_or(Ipv4Addr::new(127,0,0,1));
        .expect("Could not determine NAT external address");
    dbg!(external_address);

    let mapping = PortMapping::new(
        gateway_address,
        local_address,
        InternetProtocol::Udp,
        BIND_PORT,
        Default::default()
    ).await.expect("Could not obtain NAT port mapping");

    let bind = SocketAddr::new(local_address, BIND_PORT.get());
    let external = SocketAddr::new(IpAddr::V4(external_address), mapping.external_port().get());
    (mapping, bind, external)
}

async fn setup_crab_natpmp() -> (PortMapping, SocketAddr, SocketAddr) {
    let local_address = netdev::net::ip::get_local_ipaddr().expect("Could not determine a local address");
    dbg!(local_address);
    let gateway_address = find_gateway_address(local_address).expect("Could not determine a gateway address");
    dbg!(gateway_address);
    let external_address = crab_nat::natpmp::external_address(gateway_address, None).await
        .unwrap_or(Ipv4Addr::new(127,0,0,1));
        // .expect("Could not determine NAT external address");
    dbg!(external_address);

    let mapping = PortMapping::new(
        gateway_address,
        local_address,
        InternetProtocol::Udp,
        BIND_PORT,
        Default::default()
    ).await.expect("Could not obtain NAT port mapping");

    let bind = SocketAddr::new(local_address, BIND_PORT.get());
    let external = SocketAddr::new(IpAddr::V4(external_address), mapping.external_port().get());
    (mapping, bind, external)
}

fn find_gateway_address(local_address: IpAddr) -> Result<IpAddr, String> {
    // let gateway = netdev::get_default_gateway().expect("Could not determine a gateway");
    let gateway = netdev::get_default_gateway()?;
    let v4 = gateway.ipv4.first().map(|ip| IpAddr::V4(*ip));
    let v6 = gateway.ipv6.first().map(|ip| IpAddr::V6(*ip));
    if local_address.is_ipv4() {
        [v4, v6]
    } else {
        [v6, v4]
    }.into_iter().flatten().next().ok_or_else(|| "Could not find an ip address on default gateway".to_string())
}
