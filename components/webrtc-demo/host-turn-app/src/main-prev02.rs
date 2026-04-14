use std::net::{ SocketAddrV4, IpAddr, Ipv4Addr };
use std::num::NonZeroU16;
use std::cell::Cell;

use futures::future::{ LocalBoxFuture, FutureExt };

#[tokio::main]
async fn main() {
    // let local_address = netdev::net::ip::get_local_ipaddr().expect("Could not determine a local address");
    // test_gateway_protos().await;
    // my_futures_select_demo().await;
}

/*
use std::time::Duration;

async fn my_futures_select_demo() {
    let task = |name: &str, sleep1: u64| {
        let name = name.to_string();
        async move || {
            println!("[task {name}] begin");
            tokio::time::sleep(Duration::from_millis(sleep1)).await;
            println!("[task {name}] finished sleep #1");
            return async move || {
                println!("[task {name}] starting sleep #2");
                tokio::time::sleep(Duration::from_millis(500)).await;
                println!("[task {name}] finished sleep #2");
                println!("[task {name}] exiting");
                name
            };
        }
    };
    let taskA = task("A", 500);
    let taskB = task("B", 200);
    let (result, _, remaining) = futures::future::select_all([
        taskA().boxed_local(),
        taskB().boxed_local()
    ]).await;
    let result = result().await;
    println!("assume [task {result}] failed, try remaining");
    let (result, _, remaining) = futures::future::select_all(remaining).await;
    // dbg!(result);
    let result = result().await;
}

async fn my_futures_select_demo_old() {
    let flag_first = Cell::new(false);
    let flag_continue = Cell::new(false);
    let (result, _, remaining) = futures::future::select_all([
        (async || {
            let name = "A";
            println!("[task {name}] begin");
            tokio::time::sleep(Duration::from_millis(500)).await;
            println!("[task {name}] finished sleep #1");
            let is_first = !flag_first.replace(true);
            if is_first {
                println!("[task {name}] I'm first");
            } else {
                println!("[task {name}] I'm second, waiting...");
                futures::future::ready(()).await;
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            println!("[task {name}] starting sleep #2");
            tokio::time::sleep(Duration::from_millis(500)).await;
            println!("[task {name}] finished sleep #2");
            println!("[task {name}] exiting");
            name
        })().boxed_local(),
        (async || {
            let name = "B";
            println!("[task {name}] begin");
            tokio::time::sleep(Duration::from_millis(200)).await;
            println!("[task {name}] finished sleep #1");
            let is_first = !flag_first.replace(true);
            if is_first {
                println!("[task {name}] I'm first");
            } else {
                println!("[task {name}] I'm second, waiting...");
                futures::future::ready(()).await;
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            println!("[task {name}] starting sleep #2");
            tokio::time::sleep(Duration::from_millis(500)).await;
            println!("[task {name}] finished sleep #2");
            println!("[task {name}] exiting");
            name
        })().boxed_local()
    ]).await;
    dbg!(result);
    println!("assume [task {result}] failed, try remaining");
    let (result, _, remaining) = futures::future::select_all(remaining).await;
    dbg!(result);
}
// */

async fn test_gateway_protos() {
    let (pcp, igd) = futures::join!(pcp_init(), igd_init());
    dbg!(pcp);
    dbg!(igd);
}

// async fn natpmp_get_external(gateway_address: IpAddr) -> Result<Ipv4Addr, crab_nat::natpmp::Failure> {
//     crab_nat::natpmp::external_address(gateway_address).await
// }

// async fn igd_get_external() -> Result<(IpAddr, _)> {
//     let gateway = igd::aio::search_gateway(Default::default()).await?;
//     gateway.get_external_ip().await()
// }

// we should just attempt to map a port; if it fails, return early. but what if a router supports both IGD and natpmp?
// only one should succeed anyways;

// enum GatewayProtocol {
//     Crab(IpAddr),
//     Igd(igd::aio::Gateway)
// }

// struct InitState {
//     local_address: IpAddr,
//     gateway_protocol: GatewayProtocol,
// }

#[derive(Copy, Clone)]
enum IpProto {
    Udp,
    Tcp,
}

enum MappedPort {
    Pcp(Option<crab_nat::PortMapping>),
    Igd {
        gateway: igd::aio::Gateway,
        protocol: igd::PortMappingProtocol,
        external_port: u16,
        // external: SocketAddr,
    }
}

impl MappedPort {
    // async fn new_auto_select(local: SocketAddrV4, proto: IpProto) -> Result<(Self, SocketAddrV4), anyhow::Error> {
    //     // so we could use future::select(), and manually handle the Either Left and Right branches
    //     // or we could try to use future::select_all() and use Cell<bool> flags...

    //     // let flag_first = Cell::new(false);
    //     // let flag_continue = Cell::new(false);
    //     // let (result, _, remaining) = futures::future::select_all([
    //     //     async || {
    //     //         let (gateway_address, external_address) = pcp_init().await?;
    //     //         Ok(pcp_map(gateway_address, external_address, local, proto).await?)
    //     //     },
    //     //     async || {
    //     //         let (gateway, external_address) = igd_init().await?;
    //     //         Ok(igd_map(gateway, external_address, local, proto).await?)
    //     //     }
    //     // ]).await;

    //     // so, I'm not sure how 
    //     todo!();
    // }

    // async fn new_auto_select1(local: SocketAddrV4, proto: IpProto) -> Result<(Self, SocketAddrV4), anyhow::Error> {
    //     match futures::future::select(pcp_init(), igd_init()).await {
    //         futures::future::Either::Left((pcp, igd)) => {

    //         }
    //         futures::future::Either::Right((igd, pcp)) => {

    //         }
    //     }
    //     todo!();
    // }

    // async fn new_auto_select2(local: SocketAddrV4, proto: IpProto) -> Result<(Self, SocketAddrV4), anyhow::Error> {
    //     let (result, _, remaining) = futures::future::select_all([
    //         (async move || -> Result<_, anyhow::Error> {
    //             let (gateway_address, external_address) = pcp_init().await?;
    //             Ok((async move || -> Result<_, anyhow::Error> {
    //                 Ok(pcp_map(gateway_address, external_address, local, proto).await?)
    //             })().boxed_local())
    //         })().boxed_local(),
    //         (async move || -> Result<_, anyhow::Error> {
    //             let (gateway, external_address) = igd_init().await?;
    //             Ok((async move || -> Result<_, anyhow::Error> {
    //                 Ok(igd_map(gateway, external_address, local, proto).await?)
    //             })().boxed_local())
    //         })().boxed_local()
    //     ]).await;
    //     todo!();
    // }

    // async fn new_auto_select2_2(local: SocketAddrV4, proto: IpProto) -> Result<(Self, SocketAddrV4), (anyhow::Error, anyhow::Error)> {
    //     use std::pin::Pin;
    //     use futures::future::LocalBoxFuture;
    //     // let igd = async move || -> Result<_, anyhow::Error> {
    //     //     let (gateway, external_address) = igd_init().await?;
    //     //     Ok(Box::pin( move || (async move { Ok(igd_map(gateway, external_address, local, proto).await?) }).boxed_local()
    //     //         ) as Pin<Box<dyn FnOnce() -> LocalBoxFuture<'static, Result<(Self, SocketAddrV4), anyhow::Error>> >>)
    //     // };
    //     // let pcp = async move || -> Result<_, anyhow::Error> {
    //     //     let (gateway_address, external_address) = pcp_init().await?;
    //     //     Ok(Box::pin( move || (async move { Ok(pcp_map(gateway_address, external_address, local, proto).await?) }).boxed_local()
    //     //         ) as Pin<Box<dyn FnOnce() -> LocalBoxFuture<'static, Result<(Self, SocketAddrV4), anyhow::Error>> >>)
    //     // };
    //     // let (result, _, remaining) = futures::future::select_all([igd().boxed_local(), pcp().boxed_local()]).await;

    //     let (result, _, remaining) = futures::future::select_all::<[LocalBoxFuture<Result<_, anyhow::Error>>; _]>([
    //         async {
    //             let (gateway, external_address) = igd_init().await?;
    //             Ok(Box::new( move || (async move { Ok(igd_map(gateway, external_address, local, proto).await?) }).boxed_local()
    //                 ) as Box<dyn FnOnce() -> LocalBoxFuture<'static, Result<_, anyhow::Error> >>)
    //         }.boxed_local(),
    //         async {
    //             let (gateway_address, external_address) = pcp_init().await?;
    //             Ok(Box::new( move || (async move { Ok(pcp_map(gateway_address, external_address, local, proto).await?) }).boxed_local()
    //                 ) as Box<dyn FnOnce() -> LocalBoxFuture<'static, Result<_, anyhow::Error> >>)
    //         }.boxed_local()
    //     ]).await;
    //     let other = remaining.into_iter().next().unwrap();

    //     let e1 = match result {
    //         Ok(cb) => match cb().await {
    //             Ok(v) => return Ok(v),
    //             Err(e) => e
    //         }
    //         Err(e) => e
    //     };
    //     let e2 = match other.await {
    //         Ok(cb) => match cb().await {
    //             Ok(v) => return Ok(v),
    //             Err(e) => e
    //         }
    //         Err(e) => e
    //     };
    //     return Err((e1, e2));

    //     // match (async || result?().await)().await {
    //     // // match result().await {
    //     //     Err(e) => {
    //     //         // match (async || (other.await?)())().await {
    //     //         //     Err(e2) => return Err((e, e2)),
    //     //         //     Ok(v) => return Ok(v)
    //     //         // }
    //     //         todo!();
    //     //     },
    //     //     Ok(v) => return Ok(v)
    //     // }

    //     // let (result, _, remaining) = futures::future::select_all([
    //     //     (async move || -> Result<_, anyhow::Error> {
    //     //         let (gateway_address, external_address) = pcp_init().await?;
    //     //         let cont: Pin<Box<dyn FnOnce() -> _>> = Box::pin(async move || -> Result<_, anyhow::Error> {
    //     //             Ok(pcp_map(gateway_address, external_address, local, proto).await?)
    //     //         });
    //     //         Ok(cont)
    //     //     })().boxed_local(),
    //     //     (async move || -> Result<_, anyhow::Error> {
    //     //         let (gateway, external_address) = igd_init().await?;
    //     //         let cont: Pin<Box<dyn FnOnce() -> _>> = Box::pin(async move || -> Result<_, anyhow::Error> {
    //     //             Ok(igd_map(gateway, external_address, local, proto).await?)
    //     //         });
    //     //         Ok(cont)
    //     //     })().boxed_local()
    //     // ]).await;
    //     todo!();
    // }

    async fn new_auto(local: SocketAddrV4, proto: IpProto) -> Result<(Self, SocketAddrV4), (anyhow::Error, anyhow::Error)> {
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
        Err((e1, e2))
    }

    // attempts to find the external address with both pcp and igd; since it uses futures::join, it must wait for all timeouts
    async fn new_auto_join(local: SocketAddrV4, proto: IpProto) -> Result<(Self, SocketAddrV4), (anyhow::Error, anyhow::Error)> {
        let (pcp, igd) = futures::join!(pcp_init(), igd_init());
        let pcp_err = match pcp {
            Ok((gateway_address, external_address)) => {
                match pcp_map(gateway_address, external_address, local, proto).await {
                    Ok(v) => return Ok(v),
                    Err(e) => e,
                }
            },
            Err(e) => e,
        };
        let igd_err = match igd {
            Ok((gateway, external_address)) => {
                match igd_map(gateway, external_address, local, proto).await {
                    Ok(v) => return Ok(v),
                    Err(e) => e,
                }
            },
            Err(e) => e,
        };
        Err((pcp_err, igd_err))
    }

    async fn new_pcp(local: SocketAddrV4, proto: IpProto) -> anyhow::Result<(Self, SocketAddrV4)> {
        let (gateway_address, external_address) = pcp_init().await?;
        Ok(pcp_map(gateway_address, external_address, local, proto).await?)
    }

    async fn new_igd(local: SocketAddrV4, proto: IpProto) -> anyhow::Result<(Self, SocketAddrV4)> {
        let (gateway, external_address) = igd_init().await?;
        Ok(igd_map(gateway, external_address, local, proto).await?)
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
    let lease_duration = 60*60*4;
    let external_port = gateway.add_any_port(
        protocol,
        local,
        lease_duration,
        ""
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
