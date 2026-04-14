use std::net::{ SocketAddrV4, IpAddr, Ipv4Addr };
use std::num::NonZeroU16;
use std::cell::Cell;
use std::fmt;

use futures::future::{ LocalBoxFuture, FutureExt };

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // test_gateway_protos().await;
    // my_futures_select_demo().await;
    test_abortable2().await
    // start_primary(9317).await
    // test_igd().await
}

#[derive(Debug)]
struct Entry {
    remote_host: String,
    external_port: u16,
    protocol: &'static str,
    internal_port: u16,
    internal_client: String,
    enabled: bool,
    port_mapping_description: String,
    lease_duration: u32,
}

impl Entry {
    fn from(entry: igd::PortMappingEntry) -> Self {
        Entry {
            remote_host: entry.remote_host,
            external_port: entry.external_port,
            protocol: match entry.protocol {
                igd::PortMappingProtocol::TCP => "tcp",
                igd::PortMappingProtocol::UDP => "udp"
            },
            internal_port: entry.internal_port,
            internal_client: entry.internal_client,
            enabled: entry.enabled,
            port_mapping_description: entry.port_mapping_description,
            lease_duration: entry.lease_duration,
        }
    }
}

async fn test_igd() -> anyhow::Result<()> {
    let gateway = igd::aio::search_gateway(Default::default()).await?;
    // println!("gateway control schema: {:#?}", gateway.control_schema);
    // let external_address = gateway.get_external_ip().await?;
    let mut i = 0;
    loop {
        let entry = gateway.get_generic_port_mapping_entry(i).await?;
        println!("entry #{i}: {:#?}", Entry::from(entry));
        i+=1;
    }
    // gateway.remove_port(igd::PortMappingProtocol::TCP, 65479).await?;
    // Ok(())
}

async fn test_abortable() -> anyhow::Result<()> {
    let mut c = 0u64;
    let server = tokio::time::sleep(std::time::Duration::from_secs(5));
    let (server, abort_handle) = futures::future::abortable(server);
    ctrlc::set_handler(move || {
        println!("running ctrl-c handler...");
        println!("{c}++...");
        c+=1;
        abort_handle.abort();
    });
    println!("sleeping...");
    // tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    server.await?;
    println!("exiting...");
    Ok(())
}

async fn test_abortable2() -> anyhow::Result<()> {
    struct CleanUp;
    impl Drop for CleanUp {
        fn drop(&mut self) {
            println!("cleaning up CleanUp.");
        }
    }

    let mut c = 0u64;
    let server = async move {
        println!("server starting...");
        let cleanup = CleanUp;
        loop {
            println!("running... [{c}]");
            c += 1;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    };
    let (server, abort_handle) = futures::future::abortable(server);
    ctrlc::set_handler(move || {
        println!("recv ctrl-c, running handler...");
        abort_handle.abort();
    });
    server.await?;
    println!("exiting...");
    Ok(())
}

async fn start_primary(local_port: u16) -> anyhow::Result<()> {
    let local_address = netdev::net::ip::get_local_ipaddr().ok_or("Could not determine a local address").map_err(anyhow::Error::msg)?;
    let local_address = match local_address {
        IpAddr::V4(ip) => ip,
        IpAddr::V6(_) => {
            // TODO? would external address just be local address then? for now, just fail here.
            return Err(anyhow::Error::msg("expected local address to be ipv4, not ipv6"));
        }
    };
    let (mut mapped_port, external) = MappedPort::new_auto(SocketAddrV4::new(local_address, local_port), IpProto::Tcp).await?;
    println!("port mapping successful: {external}");

    let time_s = 15;
    let server = tokio::time::sleep(std::time::Duration::from_secs(time_s));
    let (server, abort_handle) = futures::future::abortable(server);
    ctrlc::set_handler(move || {
        println!("running ctrl-c handler...");
        abort_handle.abort();
    });
    println!("running \"server\" (mock: sleep {time_s} seconds)");
    match server.await {
        Ok(_) => (),
        Err(_) => println!("server aborted...")
    }
    match mapped_port.try_drop().await {
        Ok(()) => println!("mapped port successfully dropped."),
        Err(e) => println!("failed to drop mapped port: {e:?}")
    }
    println!("exiting...");
    Ok(())
}


// use std::sync::{ Arc, Mutex };

// async fn start_primary_bad(local_port: u16) -> anyhow::Result<()> {
//     let local_address = netdev::net::ip::get_local_ipaddr().ok_or("Could not determine a local address").map_err(anyhow::Error::msg)?;
//     let local_address = match local_address {
//         IpAddr::V4(ip) => ip,
//         IpAddr::V6(_) => {
//             // TODO? would external address just be local address then? for now, just fail here.
//             return Err(anyhow::Error::msg("expected local address to be ipv4, not ipv6"));
//         }
//     };
//     let (mut mapped_port, external) = MappedPort::new_auto(SocketAddrV4::new(local_address, local_port), IpProto::Tcp).await?;

//     let server = tokio::time::sleep(std::time::Duration::from_secs(5));
//     let (server, abort_handle) = futures::future::abortable(server);
//     ctrlc::set_handler(async move || {
//         println!("running ctrl-c handler...");
//         drop_mapped_port(handler_mapped_port);
//         abort_handle.abort();
//     });
//     println!("running \"server\" (mock: sleep 5 seconds)");
//     server.await?;
//     drop_mapped_port(mapped_port).await;
//     println!("exiting...");
//     Ok(())
// }

// async fn drop_mapped_port(mapped_port: Arc<Mutex<MappedPort>>) {
//     match mapped_port.try_lock() {
//         Ok(mut mapped_port) => match mapped_port.try_drop().await {
//             Ok(()) => {
//                 println!("mapped port successfully dropped.");
//             }
//             Err(e) => {
//                 println!("failed to drop mapped port: {e:?}");
//             }
//         }
//         Err(_) => {
//             println!("mapped_port mutex already locked; assuming it's being dropped by another thread");
//         }
//     }
// }

async fn test_port_map() -> anyhow::Result<()> {
    let local_address = netdev::net::ip::get_local_ipaddr().ok_or("Could not determine a local address").map_err(anyhow::Error::msg)?;
    let local_address = match local_address {
        IpAddr::V4(ip) => ip,
        IpAddr::V6(_) => {
            return Err(anyhow::Error::msg("local address is ipv6; port mapping not required."));
        }
    };
    let (mut mapped_port, external) = MappedPort::new_auto(SocketAddrV4::new(local_address, 9317), IpProto::Tcp).await?;
    println!("local port is now mapped to {external:?}");
    tokio::signal::ctrl_c().await?;
    mapped_port.try_drop().await?;
    println!("exiting...");
    Ok(())
}

#[derive(Copy, Clone)]
pub enum IpProto {
    Udp,
    Tcp,
}

#[derive(Debug)]
pub struct MappedPortAutoError(anyhow::Error, anyhow::Error);

impl fmt::Display for MappedPortAutoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to map port on gateway with either NAT-PMP/PCP or UPnP IGD (\"{}\", \"{}\")", self.0, self.1)
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
        let gateway_address = [v4, v6].into_iter().flatten().next().ok_or("Could not find an ip address on default gateway").map_err(anyhow::Error::msg)?;
        dbg!(gateway_address);
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
