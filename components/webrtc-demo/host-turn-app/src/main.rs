use std::net::{ SocketAddr, SocketAddrV4, IpAddr, Ipv4Addr };
use std::num::NonZeroU16;
use std::cell::Cell;
use std::fmt;
use std::sync::Arc;

use futures::future::{ LocalBoxFuture, FutureExt };

use turn_server::{ config::{Config, Turn, Interface, Transport, Api, Log, LogLevel, Auth},  };

const LOG_LEVEL: LogLevel = LogLevel::Trace;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_logger::init_with_level(LOG_LEVEL.as_level());
    start().await
    // Ok(())
}

fn default_turn_config(local_port: u16, external: SocketAddrV4) -> Config {
    Config {
        turn: Turn {
            realm: "localhost".to_string(),
            interfaces: vec![Interface {
                transport: Transport::TCP,
                // bind: SocketAddr::new(local_address, local_port),
                bind: ([0, 0, 0, 0], local_port).into(),
                external: SocketAddr::V4(external)
            }]
        },
        api: Api {
            bind: ([127, 0, 0, 1], 3003).into()
        },
        log: Log { level: LOG_LEVEL },
        auth: Auth {
            static_credentials: Default::default(),
            static_auth_secret: Some("some_password_123".to_string())
        }
    }
}

async fn start() -> anyhow::Result<()> {
    let port = 9317;
    let (mut mapped_port, external) = get_mapped_port(port).await?;
    let mut clean_up_mapped_port = async move || {
        match mapped_port.try_drop().await {
            Ok(()) => println!("removed mapped port"),
            Err(e) => println!("Error, removing mapped port failed: {e}")
        }
    };
    let config = Arc::new(default_turn_config(port, external));
    let server = turn_server::startup(config);
    let (server, server_handle) = futures::future::abortable(server);
    let result = ctrlc::set_handler(move || {
        // println!("signal ctrl-c received...");
        server_handle.abort();
    });
    match result {
        Ok(()) => (),
        Err(e) => {
            clean_up_mapped_port().await;
            return Err(e.into());
        }
    }
    match server.await {
        Ok(inner) => match inner {
            Ok(()) => (),
            Err(e) => println!("turn_server failed: {e}")
        }
        // aborted;
        Err(_) => (),
    }
    clean_up_mapped_port().await;
    println!("exiting.");
    Ok(())
}

async fn get_mapped_port(local_port: u16) -> anyhow::Result<(MappedPort, SocketAddrV4)> {
    let local_address = netdev::net::ip::get_local_ipaddr().ok_or("Could not determine a local address").map_err(anyhow::Error::msg)?;
    let local_address = match local_address {
        IpAddr::V4(ip) => ip,
        IpAddr::V6(_) => {
            // TODO? would external address just be local address then? for now, just fail here.
            return Err(anyhow::Error::msg("expected local address to be ipv4, not ipv6"));
        }
    };
    let (mapped_port, external) = MappedPort::new_auto(SocketAddrV4::new(local_address, local_port), IpProto::Tcp).await?;
    println!("created mapped port: {external}");
    Ok((mapped_port, external))
}

// async fn start_primary(local_port: u16) -> anyhow::Result<()> {
//     let local_address = netdev::net::ip::get_local_ipaddr().ok_or("Could not determine a local address").map_err(anyhow::Error::msg)?;
//     let local_address = match local_address {
//         IpAddr::V4(ip) => ip,
//         IpAddr::V6(_) => {
//             // TODO? would external address just be local address then? for now, just fail here.
//             return Err(anyhow::Error::msg("expected local address to be ipv4, not ipv6"));
//         }
//     };
//     let (mut mapped_port, external) = MappedPort::new_auto(SocketAddrV4::new(local_address, local_port), IpProto::Tcp).await?;
//     println!("port mapping successful: {external}");

//     let time_s = 15;
//     let server = tokio::time::sleep(std::time::Duration::from_secs(time_s));
//     let (server, abort_handle) = futures::future::abortable(server);
//     ctrlc::set_handler(move || {
//         println!("running ctrl-c handler...");
//         abort_handle.abort();
//     });
//     println!("running \"server\" (mock: sleep {time_s} seconds)");
//     match server.await {
//         Ok(_) => (),
//         Err(_) => println!("server aborted...")
//     }
//     match mapped_port.try_drop().await {
//         Ok(()) => println!("mapped port successfully dropped."),
//         Err(e) => println!("failed to drop mapped port: {e}")
//     }
//     println!("exiting...");
//     Ok(())
// }

#[derive(Copy, Clone)]
pub enum IpProto {
    Udp,
    Tcp,
}

#[derive(Debug)]
pub struct MappedPortAutoError(anyhow::Error, anyhow::Error);

impl fmt::Display for MappedPortAutoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to map port on gateway with either NAT-PMP/PCP or UPnP IGD;\n* 1) {};\n* 2) {};\n", self.0, self.1)
    }
}

impl std::error::Error for MappedPortAutoError {}

pub enum MappedPort {
    Pcp(Option<crab_nat::PortMapping>),
    Igd {
        gateway: igd::aio::Gateway,
        protocol: igd::PortMappingProtocol,
        external_port: u16,
        // external: SocketAddr,
    }
}

impl MappedPort {
    async fn new_auto(local: SocketAddrV4, proto: IpProto) -> Result<(Self, SocketAddrV4), MappedPortAutoError> {
        let (result, _, remaining) = futures::future::select_all::<[LocalBoxFuture<Result<_, anyhow::Error>>; _]>([
            async {
                let (gateway, external_address) = igd_init().await?;
                Ok(async move { igd_map(gateway, external_address, local, proto).await }.boxed_local())
            }.boxed_local(),
            async {
                let (gateway_address, external_address) = pcp_init().await?;
                Ok(async move { pcp_map(gateway_address, external_address, local, proto).await }.boxed_local())
            }.boxed_local()
        ]).await;
        let other = remaining.into_iter().next().unwrap();

        let e1 = match result {
            Ok(v) => match v.await {
                Ok(v) => return Ok(v),
                Err(e) => e
            }
            Err(e) => e
        };
        let e2 = match other.await {
            Ok(v) => match v.await {
                Ok(v) => return Ok(v),
                Err(e) => e
            }
            Err(e) => e
        };
        Err(MappedPortAutoError(e1, e2))
    }

    async fn new_pcp(local: SocketAddrV4, proto: IpProto) -> anyhow::Result<(Self, SocketAddrV4)> {
        let (gateway_address, external_address) = pcp_init().await?;
        Ok(pcp_map(gateway_address, external_address, local, proto).await?)
    }

    async fn new_igd(local: SocketAddrV4, proto: IpProto) -> anyhow::Result<(Self, SocketAddrV4)> {
        let (gateway, external_address) = igd_init().await?;
        Ok(igd_map(gateway, external_address, local, proto).await?)
    }

    async fn try_drop(&mut self) -> anyhow::Result<()> {
        match self {
            Self::Pcp(slot) => {
                if slot.is_none() { return Ok(()); }
                let pm = slot.take().unwrap();
                match pm.try_drop().await {
                    Ok(()) => return Ok(()),
                    Err((e, pm)) => {
                        slot.replace(pm);
                        return Err(e.into());
                    }
                }
            }
            Self::Igd { gateway, protocol, external_port } => {
                if *external_port == 0 { return Ok(()); }
                match gateway.remove_port(*protocol, *external_port).await {
                    Ok(()) => {
                        *external_port = 0;
                        Ok(())
                    },
                    Err(e) => Err(e.into())
                }
            }
        }
    }

    fn is_dropped(&self) -> bool {
        match self {
            Self::Pcp(pm) => pm.is_none(),
            Self::Igd { external_port, .. } => *external_port == 0
        }
    }
}

async fn pcp_init() -> anyhow::Result<(IpAddr, Ipv4Addr)> {
        let gateway = netdev::get_default_gateway().map_err(anyhow::Error::msg)?;
        let v4 = gateway.ipv4.first().map(|ip| IpAddr::V4(*ip));
        let v6 = gateway.ipv6.first().map(|ip| IpAddr::V6(*ip));
        // let gateway_address = if local_address.is_ipv4() {
        //     [v4, v6]
        // } else {
        //     [v6, v4]
        // }.into_iter().flatten().next().ok_or("Could not find an ip address on default gateway").map_err(anyhow::Error::msg)?;
        let gateway_address = [v4, v6].into_iter().flatten().next()
            .ok_or("Could not find an ip address on default gateway").map_err(anyhow::Error::msg)?;
        // dbg!(gateway_address);
        let external_address = crab_nat::natpmp::external_address(gateway_address, None).await?;
        Ok((gateway_address, external_address))
}

async fn pcp_map(
    gateway_address: IpAddr,
    external_address: Ipv4Addr,
    local: SocketAddrV4,
    proto: IpProto
) -> anyhow::Result<(MappedPort, SocketAddrV4)> {
    let mapping = crab_nat::PortMapping::new(
        gateway_address,
        IpAddr::V4(*local.ip()),
        match proto {
            IpProto::Udp => crab_nat::InternetProtocol::Udp,
            IpProto::Tcp => crab_nat::InternetProtocol::Tcp,
        },
        NonZeroU16::new(local.port()).ok_or("local port must not be zero").map_err(anyhow::Error::msg)?,
        // crab_nat::PortMappingOptions {
        //     timeout_config
        //     ..Default::default()
        // }
        Default::default()
    ).await?;
    let external_port = mapping.external_port().get();
    Ok((
        MappedPort::Pcp(Some(mapping)),
        SocketAddrV4::new(external_address, external_port)
    ))
}

async fn igd_init() -> anyhow::Result<(igd::aio::Gateway, Ipv4Addr)> {
    let gateway = igd::aio::search_gateway(Default::default()).await?;
    let external_address = gateway.get_external_ip().await?;
    Ok((gateway, external_address))
}

async fn igd_map(
    gateway: igd::aio::Gateway,
    external_address: Ipv4Addr,
    local: SocketAddrV4,
    proto: IpProto
) -> anyhow::Result<(MappedPort, SocketAddrV4)> {
    let protocol = match proto {
        IpProto::Udp => igd::PortMappingProtocol::UDP,
        IpProto::Tcp => igd::PortMappingProtocol::TCP
    };
    let lease_duration = 60*60;
    let external_port = gateway.add_any_port(
        protocol,
        local,
        lease_duration,
        "turn-server"
    ).await?;
    Ok((
        MappedPort::Igd {
            gateway,
            protocol,
            external_port,
        },
        SocketAddrV4::new(external_address, external_port)
    ))
}
